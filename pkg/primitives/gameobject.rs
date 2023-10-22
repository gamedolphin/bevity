use serde::{Deserialize, Serialize};

use crate::FileReference;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityGameObject {
    #[serde(default = "default_active", alias = "m_IsActive")]
    pub active: i32,

    #[serde(default, alias = "m_Component")]
    pub components: Vec<UnityComponent>,

    #[serde(default, alias = "m_Name")]
    pub name: String,
}

fn default_active() -> i32 {
    1
}

impl UnityGameObject {
    pub fn is_active(&self) -> bool {
        self.active == 1
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityComponent {
    pub component: FileReference,
}
