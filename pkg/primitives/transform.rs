use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{UnityQuaternion, UnityVector3};

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct UnityTransform {
    #[serde(rename = "m_LocalPosition")]
    pub position: UnityVector3,
    #[serde(rename = "m_LocalRotation")]
    pub rotation: UnityQuaternion,
    #[serde(rename = "m_LocalScale")]
    pub scale: UnityVector3,
}

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct UnityTransformMeta {
    pub object_id: u64,
}

#[derive(Component, Debug, Default, Copy, Clone)]
pub struct UnityTransformDirty;

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
        }
    }
}
