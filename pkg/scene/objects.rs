use bevity_primitives::UnityTransform;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum UnitySceneObject {
    Transform(UnityTransform),
    #[serde(other)]
    DontCare,
}
