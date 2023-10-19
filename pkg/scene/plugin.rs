use bevity_primitives::{
    get_transform_for_prefab, load_camera_skybox_system, FileReference, UnityMeshFilterExtra,
    UnityMeshRendererExtra, UnityMeshRequiresLoad, UnityPrefabInstance, UnityTransform,
};
use bevy::{
    ecs::{system::EntityCommands, world::EntityMut},
    prelude::*,
    utils::HashMap,
};
use std::marker::PhantomData;

use crate::{
    get_transform, ResourcesPlugin, UnityResource, UnityScene, UnitySceneObject, UnityTransformMeta,
};

#[derive(Default)]
pub struct ScenePlugin<T>(PhantomData<T>);

#[derive(Resource, Default)]
pub struct SceneResource<T> {
    pub scenes: HashMap<String, UnityScene<T>>,
    pub current: Option<String>,
}

#[derive(Component)]
pub struct ObjectLocalFileId {
    pub object_id: i64,
}

pub trait MonoBehaviour {
    fn add_component_to_entity(&self, object_id: i64, cmd: &mut EntityCommands);
    fn update_component(&self, cmd: &mut EntityMut);
}

impl<T: serde::de::DeserializeOwned + Sync + Send + 'static + Default + MonoBehaviour> Plugin
    for ScenePlugin<T>
{
    fn build(&self, app: &mut App) {
        // read build settings and parse all scenes

        app.insert_resource::<SceneResource<T>>(SceneResource::default())
            .add_plugins(ResourcesPlugin)
            .add_systems(Update, load_scene_if_changed::<T>)
            .add_systems(Update, (load_camera_skybox_system, load_unity_mesh_system));
    }
}

#[derive(Default)]
pub struct LocalSceneConfig {
    pub current_loaded: Option<String>,
}

fn load_scene_if_changed<T: Sync + Send + MonoBehaviour + 'static>(
    scene: Res<SceneResource<T>>,
    mut unity_res: ResMut<UnityResource>,
    mut local: Local<LocalSceneConfig>,
    commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let Some(current) = &scene.current else {
        return;
    };

    let loaded = match &local.current_loaded {
        Some(val) => val,
        None => "",
    };

    if loaded == current {
        return;
    }

    let Some(current_scene) = scene.scenes.get(current) else {
        tracing::error!("invalid scene name {}", current);
        return;
    };

    load_scene(current_scene, commands, &mut unity_res, &asset_server);

    local.current_loaded = Some(current.clone());
}

fn load_scene<T>(
    scene: &UnityScene<T>,
    mut commands: Commands,
    unity_res: &mut ResMut<UnityResource>,
    asset_server: &Res<AssetServer>,
) where
    T: MonoBehaviour,
{
    let render_settings = scene.get_render_settings();

    let mut object_map: HashMap<i64, Entity> = HashMap::new();
    let mut transform_map: HashMap<i64, Entity> = HashMap::new();

    if let Some(render_settings) = render_settings {
        commands.insert_resource(AmbientLight {
            color: render_settings.ambient_sky_color.into(),
            brightness: render_settings.ambient_intensity,
        });
    }

    scene
        .0
        .iter()
        .filter_map(|(id, g)| match g {
            UnitySceneObject::GameObject(g) => Some((id, g)),
            _ => None,
        })
        .for_each(|(id, game_object)| {
            let Some((comp_id, transform)) = get_transform(game_object, &scene.0) else {
                // some gameobject without a transform??
                tracing::error!(
                    "found game object without a transform: {}",
                    game_object.name
                );
                return;
            };

            let local = transform.into();

            let mut entity = commands.spawn(TransformBundle { local, ..default() });
            entity.insert(ObjectLocalFileId { object_id: *id });
            entity.insert(UnityTransformMeta { object_id: comp_id });
            entity.insert(VisibilityBundle {
                visibility: if game_object.is_active() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                ..default()
            });

            game_object
                .components
                .iter()
                .filter_map(|c| {
                    let comp = scene.0.get(&c.component.file_id)?;
                    Some((c.component.file_id, comp))
                })
                .for_each(|(object_id, comp)| {
                    comp.spawn_meta(object_id, &mut entity);
                    comp.spawn_components(
                        object_id,
                        local,
                        &render_settings,
                        unity_res,
                        &mut entity,
                    );
                });

            object_map.insert(*id, entity.id());
            transform_map.insert(comp_id, entity.id());
        });

    let prefab_transforms: HashMap<i64, (&i64, &UnityTransform)> = scene
        .0
        .iter()
        .filter_map(|(id, trx)| match trx {
            UnitySceneObject::Transform(t) => {
                if t.prefab_instance.file_id != 0 {
                    Some((t.prefab_instance.file_id, (id, t)))
                } else {
                    None
                }
            }
            _ => None,
        })
        .fold(HashMap::new(), |mut acc, (id, t)| {
            acc.insert(id, t);

            acc
        });

    scene
        .0
        .iter()
        .filter_map(|(id, game_object)| match game_object {
            UnitySceneObject::PrefabInstance(p) => Some((id, p)),
            _ => None,
        })
        .for_each(|(id, prefab)| {
            let local = get_transform_for_prefab(prefab).into();
            let mut entity = commands.spawn((TransformBundle { local, ..default() },));
            entity.insert(ObjectLocalFileId { object_id: *id });

            spawn_prefab(prefab, &mut entity, local, unity_res, asset_server);
            object_map.insert(*id, entity.id());

            let Some((transform_id, _)) = prefab_transforms.get(id) else {
                tracing::warn!("failed to get a transform for {}", id);
                return;
            };
            entity.insert(UnityTransformMeta {
                object_id: **transform_id,
            });
            transform_map.insert(**transform_id, entity.id());
        });

    scene
        .0
        .iter()
        .filter_map(|(id, game_object)| match game_object {
            UnitySceneObject::GameObject(g) => {
                if let Some((_, transform)) = get_transform(g, &scene.0) {
                    return Some((id, transform));
                };

                None
            }
            _ => None,
        })
        .filter_map(|(gameobject_id, transform)| {
            let parent = object_map.get(gameobject_id)?;
            Some((parent, transform))
        })
        .for_each(|(parent, transform)| {
            let children = transform
                .children
                .iter()
                .filter_map(|c| transform_map.get(&c.file_id).copied())
                .collect::<Vec<Entity>>();

            commands.entity(*parent).push_children(&children);
        });
}

fn spawn_prefab(
    prefab: &UnityPrefabInstance,
    cmd: &mut EntityCommands,
    transform: Transform,
    res: &mut ResMut<UnityResource>,
    asset_server: &Res<AssetServer>,
) {
    let Some(guid) = &prefab.source.guid else {
        // some prefab without a guid? thats a bug
        return;
    };

    let Some(referenced_prefab) = res.all_map.get(guid) else {
        // some unknown prefab
        return;
    };

    if referenced_prefab.ends_with(".glb") {
        spawn_gltf(
            guid,
            &referenced_prefab.clone(),
            cmd,
            transform,
            res,
            asset_server,
        );
    }
}

fn spawn_gltf(
    guid: &str,
    path: &str,
    cmd: &mut EntityCommands,
    transform: Transform,
    res: &mut ResMut<UnityResource>,
    asset_server: &Res<AssetServer>,
) {
    let scene = if let Some(model) = res.models.get(guid) {
        model.clone()
    } else {
        let path = res.base_path.join("..").join(path);
        let path = format!("{}#Scene0", path.to_string_lossy());
        tracing::info!("loading {} at {:?}", path, transform);
        let handle = asset_server.load(path);
        res.models.insert(guid.to_string(), handle.clone());

        handle
    };

    cmd.insert(SceneBundle {
        scene,
        transform,
        ..default()
    });
}

pub fn load_unity_mesh_system(
    meshes: Query<
        (
            Entity,
            &Transform,
            &UnityMeshFilterExtra,
            &UnityMeshRendererExtra,
        ),
        With<UnityMeshRequiresLoad>,
    >,
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut unity_res: ResMut<UnityResource>,
) {
    for (entity, transform, mesh_filter, mesh_renderer) in &meshes {
        let mf = &mesh_filter.mesh;
        let mesh = load_primitve_mesh(mf, &mut mesh_assets, &mut unity_res);

        let Some(material) = load_material(mesh_renderer, &unity_res) else {
            continue;
        };

        let mut cmd = commands.entity(entity);
        cmd.insert(PbrBundle {
            mesh,
            material,
            transform: *transform,
            ..default()
        });

        cmd.remove::<UnityMeshRequiresLoad>();
    }
}

fn load_material(
    mesh_renderer: &UnityMeshRendererExtra,
    unity_res: &ResMut<UnityResource>,
) -> Option<Handle<StandardMaterial>> {
    let mr = mesh_renderer.materials.first()?.guid.clone()?;
    let mat = unity_res.standard_materials.get(&mr)?;

    Some(mat.clone())
}

fn load_primitve_mesh(
    mf: &FileReference,
    mesh_assets: &mut ResMut<Assets<Mesh>>,
    loaded: &mut ResMut<UnityResource>,
) -> Handle<Mesh> {
    let unique_id = format!("{}_{}", mf.guid.clone().unwrap_or_default(), mf.file_id);
    if let Some(existing) = loaded.meshes.get(&unique_id) {
        return existing.clone();
    }

    let handle = match mf.file_id {
        // cube
        10202 => mesh_assets.add(Mesh::from(shape::Cube { size: 1.0 })),

        // plane
        10209 => mesh_assets.add(Mesh::from(shape::Plane::from_size(10.0))),
        _ => mesh_assets.add(Mesh::from(shape::UVSphere {
            radius: 1.0,
            ..default()
        })),
    };

    loaded.meshes.insert(unique_id, handle.clone());

    handle
}
