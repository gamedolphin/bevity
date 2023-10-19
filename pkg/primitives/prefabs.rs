use serde::{Deserialize, Serialize};

use crate::{FileReference, UnityTransform};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityPrefabInstance {
    #[serde(rename = "m_SourcePrefab")]
    pub source: FileReference,
    #[serde(rename = "m_Modification")]
    pub modification: PrefabModification,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PrefabModification {
    #[serde(rename = "m_TransformParent")]
    pub parent: FileReference,
    #[serde(default, rename = "m_Modifications")]
    pub modifications: Vec<ModificationProperty>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ModificationProperty {
    pub target: FileReference,
    #[serde(default, rename = "propertyPath")]
    pub path: String,
    pub value: PropertyOption,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum PropertyOption {
    #[default]
    Null,
    Bool(bool),
    Number(f32),
    String(String),
}

impl PropertyOption {
    pub fn get_number(&self) -> Option<f32> {
        match self {
            PropertyOption::Number(f) => Some(*f),
            _ => None,
        }
    }
}

pub fn get_transform_for_prefab(prefab: &UnityPrefabInstance) -> UnityTransform {
    let mut transform = UnityTransform::default();
    prefab
        .modification
        .modifications
        .iter()
        .for_each(|m| match m.path.as_str() {
            "m_LocalPosition.x" => {
                transform.position.x = m.value.get_number().unwrap_or_default();
            }
            "m_LocalPosition.y" => {
                transform.position.y = m.value.get_number().unwrap_or_default();
            }
            "m_LocalPosition.z" => {
                transform.position.z = m.value.get_number().unwrap_or_default();
            }
            "m_LocalRotation.x" => {
                transform.rotation.x = m.value.get_number().unwrap_or_default();
            }
            "m_LocalRotation.y" => {
                transform.rotation.y = m.value.get_number().unwrap_or_default();
            }
            "m_LocalRotation.z" => {
                transform.rotation.z = m.value.get_number().unwrap_or_default();
            }
            "m_LocalScale.x" => {
                transform.scale.x = m.value.get_number().unwrap_or(1.0);
            }
            "m_LocalScale.y" => {
                transform.scale.y = m.value.get_number().unwrap_or(1.0);
            }
            "m_LocalScale.z" => {
                transform.scale.z = m.value.get_number().unwrap_or(1.0);
            }
            _ => {}
        });

    transform
}
