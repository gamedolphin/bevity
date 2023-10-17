use std::f32::consts::TAU;

use bevity::{BevityPlugin, UnityChangeMap, UnityChangeObject};
use bevy::{
    ecs::{system::EntityCommands, world::EntityMut},
    prelude::*,
};
use exported::{DoRotate, DoWiggle};
use serde::{Deserialize, Serialize};

mod exported;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
#[serde(untagged)]
pub enum BevityExported {
    DoRotate(DoRotate),
    DoWiggle(DoWiggle),
    #[default]
    DontCare,
}

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct DoWiggleMeta {
    pub object_id: u64,
}

#[derive(Component, Debug, Deserialize, Clone, Serialize)]
pub struct DoRotateMeta {
    pub object_id: u64,
}

#[derive(Component)]
pub struct DoRotateDirty;
#[derive(Component)]
pub struct DoWiggleDirty;

impl Plugin for BevityExported {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (track_DoRotate, track_DoWiggle));
    }
}

fn track_DoRotate(
    query: Query<(Entity, &DoRotateMeta, &DoRotate, Option<&DoRotateDirty>), Changed<DoRotate>>,
    mut change_map: ResMut<UnityChangeMap>,
    mut commands: Commands,
) {
    for (entity, meta, val, dirty) in &query {
        if let Ok(serialized) =
            serde_json::to_string(&UnityChangeObject::<BevityExported>::MonoBehaviour(
                BevityExported::DoRotate(val.clone()),
            ))
        {
            if dirty.is_some() {
                change_map.dirty.insert(meta.object_id);
                commands.entity(entity).remove::<DoRotateDirty>();
            } else {
                change_map.changes.insert(meta.object_id, serialized);
            }
        };
    }
}

fn track_DoWiggle(
    query: Query<(Entity, &DoWiggleMeta, &DoWiggle, Option<&DoWiggleDirty>), Changed<DoWiggle>>,
    mut change_map: ResMut<UnityChangeMap>,
    mut commands: Commands,
) {
    for (entity, meta, val, dirty) in &query {
        if let Ok(serialized) =
            serde_json::to_string(&UnityChangeObject::<BevityExported>::MonoBehaviour(
                BevityExported::DoWiggle(val.clone()),
            ))
        {
            if dirty.is_some() {
                change_map.dirty.insert(meta.object_id);
                commands.entity(entity).remove::<DoWiggleDirty>();
            } else {
                change_map.changes.insert(meta.object_id, serialized);
            }
        };
    }
}

impl bevity::MonoBehaviour for BevityExported {
    fn add_component_to_entity(&self, cmd: &mut EntityCommands) {
        match self {
            BevityExported::DoRotate(d) => {
                cmd.insert(d.clone());
            }
            BevityExported::DoWiggle(b) => {
                cmd.insert(b.clone());
            }
            BevityExported::DontCare => {
                println!("found dont care component");
            }
        };
    }

    fn update_component(&self, cmd: &mut EntityMut) {
        match self {
            BevityExported::DoRotate(d) => {
                cmd.insert(d.clone());
                cmd.insert(DoRotateDirty);
            }
            BevityExported::DoWiggle(b) => {
                cmd.insert(b.clone());
                cmd.insert(DoWiggleDirty);
            }
            BevityExported::DontCare => {
                println!("found dont care component");
            }
        };
    }
}

fn main() {
    println!("starting");
    App::new()
        .add_plugins((DefaultPlugins, BevityPlugin::<BevityExported>::default()))
        .add_plugins(BevityExported::DontCare)
        .add_systems(Update, rotate_cube)
        .run();
}

fn rotate_cube(mut cubes: Query<(&mut Transform, &DoRotate)>, timer: Res<Time>) {
    for (mut transform, cube) in &mut cubes {
        // The speed is first multiplied by TAU which is a full rotation (360deg) in radians,
        // and then multiplied by delta_seconds which is the time that passed last frame.
        // In other words. Speed is equal to the amount of rotations per second.
        transform.rotate_y(cube.speed * TAU * timer.delta_seconds());
    }
}
