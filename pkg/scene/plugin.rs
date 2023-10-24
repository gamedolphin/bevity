use bevity_primitives::*;
use bevy::{
    ecs::{system::EntityCommands, world::EntityMut},
    prelude::*,
    utils::HashMap,
};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;

use crate::{
    get_transform, parse_scene_file, ResourcesPlugin, UnityRenderSettings, UnityResource,
    UnityScene, UnitySceneObject, UnityTransformMeta,
};

#[derive(Default)]
pub struct ScenePlugin<T>(PhantomData<T>);

#[derive(Resource, Default)]
pub struct SceneResource<T> {
    pub scenes: HashMap<String, UnityScene<T>>,
    pub current: Option<String>,
}

#[derive(Resource, Default)]
pub struct UnityEntityMap {
    pub object_map: HashMap<i64, Entity>,
    pub guid_map: HashMap<String, Entity>,
}

pub trait MonoBehaviour {
    fn add_component_to_entity(&self, object_id: i64, cmd: &mut EntityCommands);
    fn update_component(&self, cmd: &mut EntityMut);
}

impl<T: serde::de::DeserializeOwned + Sync + Send + 'static + Default + MonoBehaviour + Clone>
    Plugin for ScenePlugin<T>
{
    fn build(&self, app: &mut App) {
        // read build settings and parse all scenes

        app.insert_resource::<SceneResource<T>>(SceneResource::default())
            .add_plugins(ResourcesPlugin::<T>::default())
            .insert_resource(UnityEntityMap::default())
            .add_systems(Update, load_scene_if_changed::<T>)
            .insert_resource(Msaa::Off)
            .add_systems(
                Update,
                (
                    load_camera_skybox_system,
                    load_unity_mesh_system::<T>,
                    fix_gltf_mesh,
                    load_mesh_collider_system,
                ),
            );
    }
}

#[derive(Default)]
pub struct LocalSceneConfig {
    pub current_loaded: Option<String>,
}

fn load_scene_if_changed<
    T: Sync + Send + MonoBehaviour + 'static + Default + DeserializeOwned + Clone,
>(
    scene: Res<SceneResource<T>>,
    mut unity_res: ResMut<UnityResource<T>>,
    mut local: Local<LocalSceneConfig>,
    commands: Commands,
    asset_server: Res<AssetServer>,
    mut entity_map: ResMut<UnityEntityMap>,
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

    load_scene(
        current_scene,
        commands,
        &mut unity_res,
        &asset_server,
        &mut entity_map,
    );

    local.current_loaded = Some(current.clone());
}

fn load_scene<T: Sync + Send + 'static + Default + MonoBehaviour + Clone>(
    scene: &UnityScene<T>,
    mut commands: Commands,
    unity_res: &mut ResMut<UnityResource<T>>,
    asset_server: &Res<AssetServer>,
    entity_map: &mut UnityEntityMap,
) where
    T: MonoBehaviour + DeserializeOwned,
{
    let render_settings = scene.get_render_settings();
    if let Some(render_settings) = render_settings {
        commands.insert_resource(AmbientLight {
            color: render_settings.indirect_specular_color.into(),
            brightness: render_settings.ambient_intensity,
        });
    }

    let mut transform_map: HashMap<i64, Entity> = HashMap::new();

    scene
        .1
        .iter()
        .filter_map(|(id, g)| match g {
            UnitySceneObject::GameObject(g) => Some((id, g)),
            _ => None,
        })
        .for_each(|(id, game_object)| {
            let Some((comp_id, transform)) = get_transform(game_object, &scene.1) else {
                // some gameobject without a transform??
                tracing::error!(
                    "found game object without a transform: {}",
                    game_object.name
                );
                return;
            };

            let local = transform.into();

            let mut entity = commands.spawn((
                TransformBundle { local, ..default() },
                Name::new(game_object.name.clone()),
            ));
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
                    let comp = scene.1.get(&c.component.file_id)?;
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

            let guid = format!("GlobalObjectId_V1-2-{}-{}-0", &scene.0, *id);

            entity_map.object_map.insert(*id, entity.id());
            entity_map.guid_map.insert(guid, entity.id());
            transform_map.insert(comp_id, entity.id());
        });

    scene
        .1
        .iter()
        .filter_map(|(id, game_object)| match game_object {
            UnitySceneObject::PrefabInstance(p) => Some((id, p)),
            _ => None,
        })
        .for_each(|(id, prefab)| {
            let local = prefab.get_transform_for_prefab().into();
            let mut entity = commands.spawn((TransformBundle { local, ..default() },));
            entity.insert(VisibilityBundle::default());

            spawn_prefab(
                *id,
                prefab,
                local,
                &mut entity,
                unity_res,
                asset_server,
                &render_settings,
            );

            let (file_id, name) = prefab.get_local_id_and_name(*id);
            entity.insert(Name::new(name));

            // println!("creating prefab: {}", file_id);

            let guid = format!("GlobalObjectId_V1-2-{}-{}-0", &scene.0, file_id);
            entity_map.guid_map.insert(guid, entity.id());
            entity_map.object_map.insert(file_id, entity.id());

            let transform_id = prefab.get_transform_id(*id);
            // println!("creating transform id: {}", transform_id);
            entity.insert(UnityTransformMeta {
                object_id: transform_id,
            });
            transform_map.insert(transform_id, entity.id());
        });

    scene
        .1
        .iter()
        .filter_map(|(id, game_object)| match game_object {
            UnitySceneObject::GameObject(g) => {
                if let Some((_, transform)) = get_transform(g, &scene.1) {
                    return Some((id, transform));
                };

                None
            }
            _ => None,
        })
        .filter_map(|(gameobject_id, transform)| {
            let parent = entity_map.object_map.get(gameobject_id)?;
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

fn spawn_prefab<T: Sync + Send + 'static + Default + DeserializeOwned + MonoBehaviour + Clone>(
    id: i64,
    prefab: &UnityPrefabInstance,
    transform: Transform,
    cmd: &mut EntityCommands,
    res: &mut UnityResource<T>,
    asset_server: &Res<AssetServer>,
    render_settings: &Option<&UnityRenderSettings>,
) {
    let Some(guid) = &prefab.source.guid else {
        // some prefab without a guid? thats a bug
        return;
    };

    let Some(referenced_prefab) = res.all_map.get(guid) else {
        // some unknown prefab
        return;
    };

    if referenced_prefab.ends_with(".glb") || referenced_prefab.ends_with(".gltf") {
        instantiate_gltf(
            guid,
            &referenced_prefab.clone(),
            transform,
            cmd,
            res,
            asset_server,
        );

        return;
    }

    if referenced_prefab.ends_with(".prefab") {
        instantiate_prefab(
            id,
            guid,
            &referenced_prefab.clone(),
            transform,
            cmd,
            res,
            asset_server,
            render_settings,
        );
    }
}

fn instantiate_prefab<
    T: Sync + Send + 'static + Default + DeserializeOwned + MonoBehaviour + Clone,
>(
    scene_id: i64,
    guid: &str,
    path: &str,
    transform: Transform,
    cmd: &mut EntityCommands,
    res: &mut UnityResource<T>,
    asset_server: &Res<AssetServer>,
    render_settings: &Option<&UnityRenderSettings>,
) {
    let path = res.base_path.join("..").join(path);
    let prefab = res.prefabs.entry(guid.to_string()).or_insert_with(|| {
        let scene = parse_scene_file(guid, &path.to_string_lossy());
        match scene {
            Ok(scene) => scene,
            Err(e) => {
                tracing::error!("failed to parse prefab file: {}", e);
                UnityScene::default()
            }
        }
    });
    let prefab = UnityScene(prefab.0.clone(), prefab.1.clone());

    prefab
        .1
        .iter()
        .filter_map(|(id, g)| match g {
            UnitySceneObject::GameObject(g) => Some((id, g)),
            _ => None,
        })
        // id is already set by the prefab isntantiation step before this, so ignoring
        .for_each(|(_, game_object)| {
            println!("got a gameobject in the prefab!");
            game_object
                .components
                .iter()
                .filter_map(|c| {
                    let comp = prefab.1.get(&c.component.file_id)?;
                    Some((c.component.file_id, comp))
                })
                .for_each(|(object_id, comp)| {
                    let object_id = (object_id ^ scene_id) & 0x7fffffffffffffff;
                    comp.spawn_meta(object_id, cmd);
                    comp.spawn_components(object_id, transform, render_settings, res, cmd);
                });
        });

    prefab
        .1
        .iter()
        .filter_map(|(id, game_object)| match game_object {
            UnitySceneObject::PrefabInstance(p) => Some((id, p)),
            _ => None,
        })
        .for_each(|(_, p)| {
            cmd.with_children(|children| {
                let local = p.get_transform_for_prefab().into();
                let mut entity = children.spawn((TransformBundle { local, ..default() },));

                spawn_prefab(
                    scene_id,
                    p,
                    local,
                    &mut entity,
                    res,
                    asset_server,
                    render_settings,
                );
            });
        });
}

fn instantiate_gltf<T: Sync + Send + 'static + Default>(
    guid: &str,
    path: &str,
    transform: Transform,
    cmd: &mut EntityCommands,
    res: &mut UnityResource<T>,
    asset_server: &Res<AssetServer>,
) {
    let scene = if let Some(scene) = res.models.get(guid) {
        scene.clone()
    } else {
        let path = res.base_path.join("..").join(path);
        let path = format!("{}#Scene0", path.to_string_lossy());
        let handle = asset_server.load(path);
        res.models.insert(guid.to_string(), handle.clone());
        handle
    };

    let visibility = Visibility::Hidden;

    cmd.insert(SceneBundle {
        scene: scene.clone(),
        transform,
        visibility,
        ..default()
    });

    cmd.insert(FixGltfTransforms);
}

#[derive(Component)]
struct FixGltfTransforms;

fn fix_gltf_mesh(
    query: Query<Entity, With<FixGltfTransforms>>,
    children: Query<&Children>,
    mut transforms: Query<&mut Transform, With<Handle<Mesh>>>,
    mut commands: Commands,
) {
    for gltf_entity in &query {
        let mut cmd = commands.entity(gltf_entity);

        if children.iter_descendants(gltf_entity).count() == 0 {
            // not loaded yet
            continue;
        }

        for entity in children.iter_descendants(gltf_entity) {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.rotate_axis(Vec3::AXES[1], std::f32::consts::PI);
            }
        }

        println!("handling gltf model: {:?}", gltf_entity);
        cmd.insert(VisibilityBundle {
            visibility: Visibility::Inherited,
            ..default()
        });
        cmd.remove::<FixGltfTransforms>();
    }
}

// fn spawn_gltf(
//     guid: &str,
//     path: &str,
//     transform: Transform,
//     cmd: &mut EntityCommands,
//     res: &mut ResMut<UnityResource>,
//     asset_server: &Res<AssetServer>,
// ) {
//     let scene = if let Some(model) = res.models.get(guid) {
//         model.clone()
//     } else {
//         let path = res.base_path.join("..").join(path);
//         let path = format!("{}#Scene0", path.to_string_lossy());
//         let handle = asset_server.load(path);
//         res.models.insert(guid.to_string(), handle.clone());

//         handle
//     };

//     cmd.insert(SceneBundle {
//         scene,
//         transform,
//         ..default()
//     });
// }

pub fn load_unity_mesh_system<T: Sync + Send + 'static + Default>(
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
    mut unity_res: ResMut<UnityResource<T>>,
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

fn load_material<T: Sync + Send + 'static + Default>(
    mesh_renderer: &UnityMeshRendererExtra,
    unity_res: &ResMut<UnityResource<T>>,
) -> Option<Handle<StandardMaterial>> {
    let mr = mesh_renderer.materials.first()?.guid.clone()?;
    let mat = unity_res.standard_materials.get(&mr)?;

    Some(mat.clone())
}

fn load_primitve_mesh<T: Sync + Send + 'static + Default>(
    mf: &FileReference,
    mesh_assets: &mut ResMut<Assets<Mesh>>,
    loaded: &mut ResMut<UnityResource<T>>,
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
        // cylinder
        10206 => mesh_assets.add(
            shape::Cylinder {
                radius: 0.5,
                height: 2.0,
                ..default()
            }
            .into(),
        ),
        // sphere
        10207 => mesh_assets.add(
            shape::UVSphere {
                radius: 0.5,
                ..default()
            }
            .into(),
        ),
        // capsule
        10208 => mesh_assets.add(
            shape::Capsule {
                radius: 0.5,
                depth: 1.0,
                ..default()
            }
            .into(),
        ),
        _ => mesh_assets.add(Mesh::from(shape::UVSphere {
            radius: 1.0,
            ..default()
        })),
    };

    loaded.meshes.insert(unique_id, handle.clone());

    handle
}
