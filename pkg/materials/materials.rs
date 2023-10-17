use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context, Result};
use bevity_primitives::{FileReference, UnityColor, UnityVector2};
use bevity_yaml::parse_unity_yaml;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct UnityMaterial {
    #[serde(alias = "m_Name")]
    pub name: String,

    #[serde(alias = "m_Shader")]
    pub shader: FileReference,

    #[serde(alias = "m_SavedProperties")]
    pub properties: SavedProperties,
}

impl UnityMaterial {
    pub fn get_skybox_texture_id(&self) -> Option<String> {
        self.properties
            .tex_envs
            .iter()
            .find_map(|tex| tex.get("_Tex"))
            .and_then(|t| t.texture.guid.clone())
    }

    pub fn get_standard_material(&self) -> Option<StandardMaterial> {
        match self.shader.file_id {
            46 => {
                let base_color = self
                    .properties
                    .colors
                    .iter()
                    .find_map(|c| c.get("_Color"))?
                    .into();
                let emissive = self
                    .properties
                    .colors
                    .iter()
                    .find_map(|c| c.get("_EmissionColor"))?
                    .into();

                let metallic = self
                    .properties
                    .floats
                    .iter()
                    .find_map(|f| f.get("_Metallic"))?
                    .to_owned();
                Some(StandardMaterial {
                    base_color,
                    emissive,
                    metallic,
                    ..default()
                })
            }
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct SavedProperties {
    #[serde(alias = "serializedVersion")]
    pub serialized_version: u64,

    #[serde(alias = "m_TexEnvs")]
    pub tex_envs: Vec<HashMap<String, TextureInfo>>,

    #[serde(alias = "m_Floats")]
    pub floats: Vec<HashMap<String, f32>>,

    #[serde(alias = "m_Colors")]
    pub colors: Vec<HashMap<String, UnityColor>>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TextureInfo {
    #[serde(alias = "m_Texture")]
    pub texture: FileReference,
    #[serde(alias = "m_Scale")]
    pub scale: UnityVector2,
    #[serde(alias = "m_Offset")]
    pub offset: UnityVector2,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
enum MaterialContainer {
    Material(UnityMaterial),
    #[serde(other)]
    DontCare,
}

pub fn read_materials(base: &Path, config_file: &Path) -> Result<HashMap<String, UnityMaterial>> {
    let materials =
        std::fs::read_to_string(config_file).context("failed to read materials.json")?;
    let materials: HashMap<String, String> =
        serde_json::from_str(&materials).context("failed to parse materials.json")?;

    materials
        .into_iter()
        .try_fold(HashMap::new(), |mut acc, (k, v)| {
            let path = base.join("..").join(v);
            let contents = std::fs::read_to_string(path)?;
            let mat = read_single_material(&contents).context("failed to read single material")?;

            acc.insert(k, mat);

            Ok(acc)
        })
}

fn read_single_material(contents: &str) -> Result<UnityMaterial> {
    let map = parse_unity_yaml(contents)?;

    let (_, output) = map.into_iter().next().context("0 items in material file")?;

    let MaterialContainer::Material(mat) = output else {
        bail!("invalid material file");
    };

    Ok(mat)
}
