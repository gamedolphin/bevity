use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct UnityColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<UnityColor> for Color {
    fn from(value: UnityColor) -> Self {
        Color::Rgba {
            red: value.r,
            green: value.g,
            blue: value.b,
            alpha: value.a,
        }
    }
}

impl From<&UnityColor> for Color {
    fn from(value: &UnityColor) -> Self {
        Color::Rgba {
            red: value.r,
            green: value.g,
            blue: value.b,
            alpha: value.a,
        }
    }
}
