use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum ProjectSettings {
    PlayerSettings(PlayerSettings),
    #[serde(other)]
    DontCare,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PlayerSettings {
    #[serde(alias = "productName")]
    pub product_name: String,
}
