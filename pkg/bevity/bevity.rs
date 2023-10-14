use bevy::prelude::*;
use editor::EditorPlugin;
use settings::SettingsPlugin;

mod editor;
mod settings;

pub struct BevityPlugin;

impl Plugin for BevityPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EditorPlugin, SettingsPlugin));
    }
}
