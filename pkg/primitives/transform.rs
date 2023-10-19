use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{FileReference, UnityQuaternion, UnityVector3};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnityTransform {
    #[serde(default, rename = "m_LocalPosition")]
    pub position: UnityVector3,
    #[serde(default, rename = "m_LocalRotation")]
    pub rotation: UnityQuaternion,
    #[serde(default = "default_scale", rename = "m_LocalScale")]
    pub scale: UnityVector3,
    #[serde(default, rename = "m_Children", skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<FileReference>,
    #[serde(default, rename = "m_PrefabInstance", skip_serializing)]
    pub prefab_instance: FileReference,
}

impl Default for UnityTransform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: UnityVector3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            children: Default::default(),
            prefab_instance: Default::default(),
        }
    }
}

impl From<&UnityTransform> for Transform {
    fn from(value: &UnityTransform) -> Self {
        let mut pos: Vec3 = value.position.into();
        pos.z = -pos.z; // invert the z axis (left handed vs right handed)

        let mut rot: Quat = value.rotation.into();
        rot.z = -rot.z;
        rot.w = -rot.w;

        Transform::from_translation(pos)
            .with_rotation(rot)
            .with_scale(value.scale.into())
    }
}

impl From<UnityTransform> for Transform {
    fn from(value: UnityTransform) -> Self {
        let mut pos: Vec3 = value.position.into();
        pos.z = -pos.z; // invert the z axis (left handed vs right handed)

        let mut rot: Quat = value.rotation.into();
        rot.z = -rot.z;
        rot.w = -rot.w;

        Transform::from_translation(pos)
            .with_rotation(rot)
            .with_scale(value.scale.into())
    }
}

impl From<&Transform> for UnityTransform {
    fn from(value: &Transform) -> Self {
        let mut position: UnityVector3 = value.translation.into();
        position.z = -position.z; // invert the z axis (left handed vs right handed)

        let mut rotation: UnityQuaternion = value.rotation.into();
        rotation.z = -rotation.z;
        rotation.w = -rotation.w;

        UnityTransform {
            position,
            rotation,
            scale: value.scale.into(),
            children: vec![],
            prefab_instance: FileReference::default(),
        }
    }
}

fn default_scale() -> UnityVector3 {
    UnityVector3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    }
}
