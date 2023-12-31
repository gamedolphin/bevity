use std::collections::HashMap;

use bevity_primitives::*;
use bevy_atmosphere::prelude::AtmosphereCamera;
use serde::{Deserialize, Serialize};

use crate::{MonoBehaviour, UnityRenderSettings, UnityResource};

bevity_generator::inbuilt_component_list!((
    GameObject,
    Transform,
    Camera,
    Light,
    MeshFilter,
    MeshRenderer,
    PrefabInstance,
    MeshCollider,
    BoxCollider,
    SphereCollider,
    CapsuleCollider,
    Rigidbody,
    RenderSettings
));

pub fn get_transform<'a, T>(
    game_object: &'a UnityGameObject,
    scene: &'a HashMap<i64, UnitySceneObject<T>>,
) -> Option<(i64, &'a UnityTransform)> {
    game_object.components.iter().find_map(|c| {
        let comp = scene.get(&c.component.file_id)?;

        let UnitySceneObject::Transform(t) = comp else {
            return None;
        };

        Some((c.component.file_id, t))
    })
}

impl<T: MonoBehaviour + Sync + Send + 'static + Default> UnitySceneObject<T> {
    pub fn spawn_components(
        &self,
        object_id: i64,
        transform: bevy::prelude::Transform,
        render_settings: &Option<&UnityRenderSettings>,
        unity_res: &UnityResource<T>,
        commands: &mut bevy::ecs::system::EntityCommands,
    ) {
        match self {
            UnitySceneObject::Camera(c) => {
                let skybox = render_settings
                    .and_then(|r| r.skybox_material.guid.clone())
                    .and_then(|guid| unity_res.materials_map.get(&guid))
                    .and_then(|mat| mat.get_skybox_texture_id())
                    .and_then(|tex_id| unity_res.textures.get(&tex_id));

                c.add_camera_bundle(transform, skybox, commands);
                if skybox.is_none() {
                    commands.insert(AtmosphereCamera::default());
                }
            }
            UnitySceneObject::Light(l) => l.add_light_bundle(transform, commands),
            UnitySceneObject::MeshFilter(mf) => mf.add_mesh_filter_meta(commands),
            UnitySceneObject::MeshRenderer(mr) => mr.add_mesh_renderer_meta(commands),
            UnitySceneObject::BoxCollider(b) => b.add_box_collider(&transform, commands),
            UnitySceneObject::SphereCollider(sphere_collider) => {
                sphere_collider.add_sphere_collider(&transform, commands)
            }
            UnitySceneObject::CapsuleCollider(capsule_collider) => {
                capsule_collider.add_capsule_collider(&transform, commands)
            }
            UnitySceneObject::MeshCollider(mesh_collider) => {
                mesh_collider.add_mesh_collider(commands)
            }
            UnitySceneObject::Rigidbody(rb) => rb.add_rigidbody(commands),
            UnitySceneObject::MonoBehaviour(v) => v.add_component_to_entity(object_id, commands),
            _ => {}
        };
    }
}
