use std::collections::HashMap;

use anyhow::Result;
use serde::de::DeserializeOwned;

pub fn parse_unity_yaml_file<T: DeserializeOwned>(file_path: &str) -> Result<HashMap<u64, T>> {
    let file = std::fs::read_to_string(file_path)?;
    parse_unity_yaml(&file)
}

pub fn parse_unity_yaml<T: DeserializeOwned>(file: &str) -> Result<HashMap<u64, T>> {
    let file = cleanup_unity_yaml(file)?;
    let parse: HashMap<u64, T> = serde_yaml::from_str(&file)?;

    Ok(parse)
}

fn cleanup_unity_yaml(yaml: &str) -> Result<String> {
    let lines: Vec<String> = yaml
        .lines()
        .filter_map(|line| {
            if line.starts_with("%YAML") || line.starts_with("%TAG") {
                // unity specific headers. SKIP!
                None
            } else if line.starts_with("--- !u!") {
                // unity object id declared on this line
                // --- !u!104 &2 => 104 is object type and 2 is object id
                let mut splits = line.split_whitespace();
                let object_id: u64 = splits
                    .find(|&part| part.starts_with('&'))
                    .and_then(|num| num[1..].parse().ok())?;

                Some(format!("{}:", object_id))
            } else if line.starts_with(' ') {
                Some(line.to_string())
            } else {
                Some(format!("  object_type: {}", line.replace(':', "")))
            }
        })
        .collect();

    let mut lines = lines.join("\n");

    lines.push('\n'); // insert new line at the end

    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skip_headers() -> Result<()> {
        let input = r#"%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!29 &1
OcclusionCullingSettings:
"#;

        let expected = r#"1:
  object_type: OcclusionCullingSettings
"#;
        let output = cleanup_unity_yaml(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_parse_object_id() -> Result<()> {
        let input = r#"--- !u!104 &2
RenderSettings:
"#;

        let expected = r#"2:
  object_type: RenderSettings
"#;
        let output = cleanup_unity_yaml(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_preserve_indents() -> Result<()> {
        let input = r#"--- !u!104 &2
RenderSettings:
  m_Fog: 0
  m_FogColor: {r: 0.5, g: 0.5, b: 0.5, a: 1}
"#;

        let expected = r#"2:
  object_type: RenderSettings
  m_Fog: 0
  m_FogColor: {r: 0.5, g: 0.5, b: 0.5, a: 1}
"#;

        let output = cleanup_unity_yaml(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_full_example() -> Result<()> {
        let input = r#"%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
--- !u!29 &1
OcclusionCullingSettings:
  m_ObjectHideFlags: 0
--- !u!104 &2
RenderSettings:
  m_Fog: 0
"#;

        let expected = r#"1:
  object_type: OcclusionCullingSettings
  m_ObjectHideFlags: 0
2:
  object_type: RenderSettings
  m_Fog: 0
"#;

        let output = cleanup_unity_yaml(input)?;
        assert_eq!(output, expected);
        Ok(())
    }

    #[test]
    fn test_malformed_input() -> Result<()> {
        let input = r#"%YAML 1.1
%TAG !u! tag:unity3d.com,2011:
OcclusionCullingSettings:
  m_ObjectHideFlags: 0
"#;

        let expected = r#"  object_type: OcclusionCullingSettings
  m_ObjectHideFlags: 0
"#;

        let output = cleanup_unity_yaml(input)?;
        assert_eq!(output, expected);
        Ok(())
    }
}
