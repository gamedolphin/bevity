use serde::{Deserialize, Serialize};

use crate::FileReference;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityGameObject {
    #[serde(alias = "m_IsActive")]
    pub active: i32,

    #[serde(alias = "m_Component")]
    pub components: Vec<UnityComponent>,

    #[serde(alias = "m_Name")]
    pub name: String,
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
