use std::collections::HashMap;

use anyhow::Result;
use bevity_yaml::{parse_unity_yaml, parse_unity_yaml_file};
pub use objects::UnitySceneObject;

mod objects;
mod render;

pub use objects::*;
pub use render::*;

#[derive(Default, Clone)]
pub struct UnityScene<T>(pub HashMap<u64, UnitySceneObject<T>>);

impl<T> UnityScene<T> {
    pub fn get_render_settings(&self) -> Option<&RenderSettings> {
        self.0.iter().find_map(|(_, c)| match c {
            UnitySceneObject::RenderSettings(r) => Some(r),
            _ => None,
        })
    }
}

pub fn parse_scene_file<T: serde::de::DeserializeOwned>(file: &str) -> Result<UnityScene<T>> {
    let parsed = parse_unity_yaml_file(file)?;
    Ok(UnityScene(parsed))
}

pub fn parse_scene<T: serde::de::DeserializeOwned>(scene: &str) -> Result<UnityScene<T>> {
    let parsed = parse_unity_yaml(scene)?;
    Ok(UnityScene(parsed))
}

#[cfg(test)]
mod tests {
    use anyhow::bail;

    use super::*;

    #[test]
    fn test_parse_transform() -> Result<()> {
        let yaml_input = r#"%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!81 &963194226
AudioListener:
  m_ObjectHideFlags: 0
  m_CorrespondingSourceObject: {fileID: 0}
  m_PrefabInstance: {fileID: 0}
  m_PrefabAsset: {fileID: 0}
  m_GameObject: {fileID: 963194225}
  m_Enabled: 1
--- !u!4 &963194228
Transform:
  m_LocalRotation: {x: 0.35355338, y: 0.35355338, z: -0.1464466, w: 0.8535535}
  m_LocalPosition: {x: -10, y: 15, z: -10}
  m_LocalScale: {x: 1, y: 1, z: 1}
--- !u!33 &1908984289
MeshFilter:
  m_ObjectHideFlags: 0
  m_CorrespondingSourceObject: {fileID: 0}
  m_PrefabInstance: {fileID: 0}
  m_PrefabAsset: {fileID: 0}
  m_GameObject: {fileID: 1908984286}
  m_Mesh: {fileID: 10202, guid: 0000000000000000e000000000000000, type: 0}
"#;

        let parsed = parse_scene::<()>(yaml_input)?;

        assert!(parsed.0.contains_key(&963194228u64));

        if let Some(UnitySceneObject::Transform(transform)) = parsed.0.get(&963194228u64) {
            assert_eq!(transform.rotation.x, 0.35355338);
            assert_eq!(transform.rotation.y, 0.35355338);
            assert_eq!(transform.rotation.z, -0.1464466);
            assert_eq!(transform.rotation.w, 0.8535535);
            assert_eq!(transform.position.x, -10.0);
            assert_eq!(transform.position.y, 15.0);
            assert_eq!(transform.position.z, -10.0);
            assert_eq!(transform.scale.x, 1.0);
            assert_eq!(transform.scale.y, 1.0);
            assert_eq!(transform.scale.z, 1.0);
        } else {
            bail!("Expected a Transform object")
        }

        assert!(parsed.0.contains_key(&1908984289));

        if let Some(UnitySceneObject::MeshFilter(m)) = parsed.0.get(&1908984289) {
            assert_eq!(m.mesh.file_id, 10202);
            assert_eq!(m.mesh.guid, None);
        } else {
            bail!("Expected a Transform object")
        }

        Ok(())
    }

    #[test]
    fn test_malformed_transform() -> Result<()> {
        let yaml_input = r#"%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!4 &963194228
Transform:
  m_LocalRotation: {x: 0.35355338, z: -0.1464466, w: 0.8535535}
  m_LocalPosition: {x: -10, y: 15, z: -10}
  m_LocalScale: {x: 1, y: 1, z: 1}
"#;

        let _ = parse_scene::<()>(yaml_input);

        Ok(())
    }
}
