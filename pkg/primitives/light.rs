use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

use crate::UnityColor;

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct UnityLight {
    #[serde(rename = "m_Enabled")]
    pub enabled: i32,

    #[serde(rename = "m_Type")]
    pub light_type: i32,

    #[serde(rename = "m_Color")]
    pub color: UnityColor,

    #[serde(rename = "m_Intensity")]
    pub intensity: f32,

    #[serde(rename = "m_Shadows")]
    pub shadows: Shadows,
}

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct Shadows {
    #[serde(rename = "m_Type")]
    pub shadow_type: i32,
}

#[derive(Component, Debug, Clone)]
pub struct UnityLightMeta {
    object_id: u64,
}

impl UnityLight {
    pub fn add_light_bundle(
        &self,
        object_id: u64,
        transform: Transform,
        commands: &mut EntityCommands,
    ) {
        commands.insert(UnityLightMeta { object_id });

        match self.light_type {
            1 => {
                commands.insert(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        color: self.color.into(),
                        shadows_enabled: self.shadows.shadow_type != 0,
                        ..default()
                    },
                    transform,
                    ..default()
                });
            }
            _ => {}
        }
    }
}
