use serde::{Deserialize, Serialize};

use crate::FileReference;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityGameObject {
    #[serde(alias = "m_Component")]
    pub components: Vec<UnityComponent>,

    #[serde(alias = "m_Name")]
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityComponent {
    pub component: FileReference,
}
