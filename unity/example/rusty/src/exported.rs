use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct DoRotate {
    pub speed: f32,
}

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct DoWiggle {
    pub wiggle_speed: f32,
}
