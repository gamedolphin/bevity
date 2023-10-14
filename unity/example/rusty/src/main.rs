use bevity::BevityPlugin;
use bevy::prelude::*;

fn main() {
    App::new().add_plugins((DefaultPlugins, BevityPlugin)).run();
}
