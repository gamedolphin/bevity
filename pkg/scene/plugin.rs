use bevity_primitives::{
    load_camera_skybox_system, FileReference, UnityMeshFilterExtra, UnityMeshRendererExtra,
    UnityMeshRequiresLoad,
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

#[derive(Resource, Default)]
pub struct UnityEntityMap {
    pub object_map: HashMap<u64, Entity>,
}

pub trait MonoBehaviour {
    fn add_component_to_entity(&self, object_id: u64, cmd: &mut EntityCommands);
    fn update_component(&self, cmd: &mut EntityMut);
}

impl<T: serde::de::DeserializeOwned + Sync + Send + 'static + Default + MonoBehaviour> Plugin
    for ScenePlugin<T>
{
    fn build(&self, app: &mut App) {
        // read build settings and parse all scenes

        app.insert_resource::<SceneResource<T>>(SceneResource::default())
            .add_plugins(ResourcesPlugin)
            .insert_resource(UnityEntityMap::default())
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
    unity_res: Res<UnityResource>,
    map_res: ResMut<UnityEntityMap>,
    mut local: Local<LocalSceneConfig>,
    commands: Commands,
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

    load_scene(current_scene, commands, unity_res, map_res);

    local.current_loaded = Some(current.clone());
}

fn load_scene<T>(
    scene: &UnityScene<T>,
    mut commands: Commands,
    unity_res: Res<UnityResource>,
    mut map_res: ResMut<UnityEntityMap>,
) where
    T: MonoBehaviour,
{
    let render_settings = scene.get_render_settings();

    let mut transform_map: HashMap<u64, Entity> = HashMap::new();

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
            entity.insert(UnityTransformMeta { object_id: comp_id });

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
                        &unity_res,
                        &mut entity,
                    );
                });

            map_res.object_map.insert(*id, entity.id());
            transform_map.insert(comp_id, entity.id());
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
        .for_each(|(gameobject_id, transform)| {
            let parent = map_res.object_map.get(gameobject_id).unwrap();
            let children = transform
                .children
                .iter()
                .filter_map(|c| transform_map.get(&c.file_id).copied())
                .collect::<Vec<Entity>>();

            commands.entity(*parent).push_children(&children);
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
