use std::collections::HashMap;

use crate::{FileReference, UnityColor, UnityVector2};
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
