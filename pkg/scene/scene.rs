use std::collections::HashMap;

use anyhow::Result;
use bevity_yaml::{parse_unity_yaml, parse_unity_yaml_file};
use objects::UnitySceneObject;

mod objects;

pub fn parse_scene_file(file: &str) -> Result<HashMap<u64, UnitySceneObject>> {
    parse_unity_yaml_file(file)
}

pub fn parse_scene(scene: &str) -> Result<HashMap<u64, UnitySceneObject>> {
    parse_unity_yaml(scene)
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
"#;

        let parsed = parse_scene(yaml_input)?;

        assert!(parsed.contains_key(&963194228u64));

        if let Some(UnitySceneObject::Transform(transform)) = parsed.get(&963194228u64) {
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

        let _ = parse_scene(yaml_input);

        Ok(())
    }
}
