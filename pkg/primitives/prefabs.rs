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

impl UnityPrefabInstance {
    pub fn get_local_id(&self, object_id: i64) -> i64 {
        let gameobject_id = self
            .modification
            .modifications
            .iter()
            .find_map(|m| match m.path.as_str() {
                "m_Name" => Some(m.target.file_id),
                _ => None,
            })
            .unwrap();
        // https://uninomicon.com/globalobjectid
        (object_id ^ gameobject_id) & 0x7fffffffffffffff
    }

    pub fn get_transform_id(&self, object_id: i64) -> i64 {
        let transform_id = self
            .modification
            .modifications
            .iter()
            .find_map(|m| match m.path.as_str() {
                "m_LocalPosition.x" => Some(m.target.file_id),
                _ => None,
            })
            .unwrap();

        (object_id ^ transform_id) & 0x7fffffffffffffff
    }

    pub fn get_transform_for_prefab(&self) -> UnityTransform {
        let mut transform = UnityTransform::default();
        self.modification
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
                "m_LocalRotation.w" => {
                    transform.rotation.w = m.value.get_number().unwrap_or_default();
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
}
