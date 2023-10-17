use bevy::prelude::*;
use serde::{Deserialize, Serialize};

bevity::exported_component_list!((DoRotate, DoWiggle));

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct DoRotate {
    pub speed: f32,
}

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct DoWiggle {
    pub wiggle_speed: f32,
}
