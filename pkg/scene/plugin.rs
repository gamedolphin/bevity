use bevity_primitives::{
    load_camera_skybox_system, FileReference, UnityMeshFilterExtra, UnityMeshRendererExtra,
    UnityMeshRequiresLoad, UnityPrefabInstance,
};
use bevy::{
    ecs::{system::EntityCommands, world::EntityMut},
    gltf::GltfNode,
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
    pub object_map: HashMap<i64, Entity>,
    pub guid_map: HashMap<String, Entity>,
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
            .insert_resource(UnityEntityMap::default())
            .add_systems(Update, load_scene_if_changed::<T>)
            .add_systems(
                Update,
                (
                    load_camera_skybox_system,
                    load_unity_mesh_system,
                    fix_gltf_mesh,
                ),
            );
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

fn load_scene<T>(
    scene: &UnityScene<T>,
    mut commands: Commands,
    unity_res: &mut ResMut<UnityResource>,
    asset_server: &Res<AssetServer>,
    entity_map: &mut UnityEntityMap,
) where
    T: MonoBehaviour,
{
    let render_settings = scene.get_render_settings();

    let mut transform_map: HashMap<i64, Entity> = HashMap::new();

    if let Some(render_settings) = render_settings {
        commands.insert_resource(AmbientLight {
            color: render_settings.indirect_specular_color.into(),
            brightness: render_settings.ambient_intensity,
        });
    }

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

            let mut entity = commands.spawn(TransformBundle { local, ..default() });
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

            spawn_prefab(prefab, local, &mut entity, unity_res, asset_server);

            let file_id = prefab.get_local_id(*id);

            println!("creating prefab: {}", file_id);

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

fn spawn_prefab(
    prefab: &UnityPrefabInstance,
    transform: Transform,
    cmd: &mut EntityCommands,
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

    if referenced_prefab.ends_with(".glb") || referenced_prefab.ends_with(".gltf") {
        instantiate_gltf(
            guid,
            &referenced_prefab.clone(),
            transform,
            cmd,
            res,
            asset_server,
            true,
        );
    }
}

// fn fixup_gltf(
//     mut ev_asset: EventReader<AssetEvent<GltfNode>>,
//     mut assets: ResMut<Assets<GltfNode>>,
// ) {
//     for ev in ev_asset.iter() {
//         match ev {
//             AssetEvent::Created { handle } => {
//                 let node = assets.get_mut(handle).unwrap();
//                 // ^ unwrap is OK, because we know it is loaded now

//                 node.transform
//                     .rotate_axis(Vec3::AXES[2], std::f32::consts::PI);
//             }
//             _ => {}
//         }
//     }
// }

fn instantiate_gltf(
    guid: &str,
    path: &str,
    transform: Transform,
    cmd: &mut EntityCommands,
    res: &mut ResMut<UnityResource>,
    asset_server: &Res<AssetServer>,
    add_fixup: bool,
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

    let visibility = if add_fixup {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    cmd.insert(SceneBundle {
        scene: scene.clone(),
        transform,
        visibility,
        ..default()
    });

    if add_fixup {
        cmd.insert(FixGltfTransforms);
    }
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
