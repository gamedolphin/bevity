use std::{collections::HashMap, marker::PhantomData};

use bevity_primitives::{
    load_camera_skybox_system, FileReference, UnityMeshDirty, UnityMeshFilterMeta,
    UnityMeshRendererMeta, UnityTransformMeta,
};
use bevity_scene::{get_transform, RenderSettings, UnityScene, UnitySceneObject};
use bevy::{
    ecs::{system::EntityCommands, world::EntityMut},
    prelude::*,
};

use crate::resources::UnityResource;

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

impl<T: serde::de::DeserializeOwned + Sync + Send + 'static + Default + MonoBehaviour> Plugin
    for ScenePlugin<T>
{
    fn build(&self, app: &mut App) {
        // read build settings and parse all scenes

        app.insert_resource::<SceneResource<T>>(SceneResource::default())
            .insert_resource(UnityEntityMap::default())
            .add_systems(Update, load_scene_if_changed::<T>)
            .add_systems(Update, (load_camera_skybox_system, load_unity_mesh_system));
    }
}

pub trait MonoBehaviour {
    fn add_component_to_entity(&self, cmd: &mut EntityCommands);
    fn update_component(&self, cmd: &mut EntityMut);
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
                    spawn_component(
                        object_id,
                        comp,
                        local,
                        &render_settings,
                        &mut entity,
                        &unity_res,
                    )
                });

            map_res.object_map.insert(*id, entity.id());
        });
}

fn spawn_component<T>(
    object_id: u64,
    comp: &UnitySceneObject<T>,
    transform: Transform,
    render_settings: &Option<&RenderSettings>,
    commands: &mut EntityCommands,
    unity_res: &Res<UnityResource>,
) where
    T: MonoBehaviour,
{
    match comp {
        UnitySceneObject::Camera(c) => {
            let skybox = render_settings
                .and_then(|r| r.skybox_material.guid.clone())
                .and_then(|guid| unity_res.materials_map.get(&guid))
                .and_then(|mat| mat.get_skybox_texture_id())
                .and_then(|tex_id| unity_res.textures.get(&tex_id));

            c.add_camera_bundle(object_id, transform, skybox, commands);
        }
        UnitySceneObject::Light(l) => l.add_light_bundle(object_id, transform, commands),
        UnitySceneObject::MeshFilter(mf) => mf.add_mesh_filter_meta(object_id, commands),
        UnitySceneObject::MeshRenderer(mr) => mr.add_mesh_renderer_meta(object_id, commands),
        UnitySceneObject::MonoBehaviour(v) => v.add_component_to_entity(commands),
        _ => {}
    };
}

pub fn load_unity_mesh_system(
    meshes: Query<
        (
            Entity,
            &Transform,
            &UnityMeshFilterMeta,
            &UnityMeshRendererMeta,
        ),
        With<UnityMeshDirty>,
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

        cmd.remove::<UnityMeshDirty>();
    }
}

fn load_material(
    mesh_renderer: &UnityMeshRendererMeta,
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
