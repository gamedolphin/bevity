use bevy::prelude::*;
use serde::{Deserialize, Serialize};

bevity::exported_component_list!((DoRotate, DoWiggle, PlayerCharacter));

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct DoRotate {
    pub speed: f32,
}

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct DoWiggle {
    pub wiggle_speed: f32,
}

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct PlayerCharacter {
    pub move_speed: f32,
    pub float_height: f32,
    pub jump_height: f32,
}
