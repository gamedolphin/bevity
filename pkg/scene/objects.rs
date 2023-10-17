use std::collections::HashMap;

use bevity_primitives::*;
use serde::{Deserialize, Serialize};

use crate::RenderSettings;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "object_type")]
pub enum UnitySceneObject<T> {
    GameObject(UnityGameObject),
    Transform(UnityTransform),
    Camera(UnityCamera),
    Light(UnityLight),
    MeshFilter(UnityMeshFilter),
    MeshRenderer(UnityMeshRenderer),
    MonoBehaviour(T),
    RenderSettings(RenderSettings),
    #[serde(other)]
    DontCare,
}

pub fn get_transform<'a, T>(
    game_object: &'a UnityGameObject,
    scene: &'a HashMap<u64, UnitySceneObject<T>>,
) -> Option<(u64, &'a UnityTransform)> {
    game_object.components.iter().find_map(|c| {
        let comp = scene.get(&c.component.file_id)?;

        let UnitySceneObject::Transform(t) = comp else {
            return None;
        };

        Some((c.component.file_id, t))
    })
}
