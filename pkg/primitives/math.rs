use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct UnityVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl UnityVector3 {
    pub fn transform_coordinates(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: -self.z,
        }
    }
}

impl From<UnityVector3> for Vec3 {
    fn from(value: UnityVector3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Vec3> for UnityVector3 {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct UnityQuaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<UnityQuaternion> for Quat {
    fn from(value: UnityQuaternion) -> Self {
        Quat::from_array([value.x, value.y, value.z, value.w])
    }
}

impl From<Quat> for UnityQuaternion {
    fn from(value: Quat) -> Self {
        UnityQuaternion {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct UnityVector2 {
    pub x: f32,
    pub y: f32,
}
