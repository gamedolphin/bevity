use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UnityTransform {
    #[serde(alias = "m_LocalPosition")]
    pub position: UnityVector3,
    #[serde(alias = "m_LocalRotation")]
    pub rotation: UnityQuaternion,
    #[serde(alias = "m_LocalScale")]
    pub scale: UnityVector3,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnityVector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnityQuaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnityVector2 {
    pub x: f32,
    pub y: f32,
}
