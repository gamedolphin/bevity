use anyhow::{bail, Context, Result};
use bevity_yaml::parse_unity_yaml;
use bevy::prelude::*;
use std::path::Path;

mod player;

pub use player::*;

pub fn parse_project_settings(settings: &str) -> Result<player::PlayerSettings> {
    let map = parse_unity_yaml(settings)?;

    let (_, output) = map
        .into_iter()
        .next()
        .context("0 items in project settings")?;

    let ProjectSettings::PlayerSettings(settings) = output else {
        bail!("invalid project settings found")
    };

    Ok(settings)
}

pub fn parse_project_settings_file(base: &Path) -> Result<player::PlayerSettings> {
    let file = base.join("ProjectSettings/ProjectSettings.asset");
    let contents = std::fs::read_to_string(file)?;

    parse_project_settings(&contents)
}

#[derive(Default)]
pub struct SettingsPlugin;

#[derive(Resource, Debug, Default)]
pub struct UnitySettings {
    pub player: PlayerSettings,
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let path = bevity_yaml::get_assets_dir();

        let path = Path::new(&path).join("..");
        let Ok(player) = parse_project_settings_file(&path) else {
            tracing::error!("failed to parse project settings");
            return;
        };

        app.insert_resource(UnitySettings { player })
            .add_systems(Startup, window_settings_system);
    }
}

fn window_settings_system(mut windows: Query<&mut Window>, settings: Res<UnitySettings>) {
    let Ok(mut window) = windows.get_single_mut() else {
        // window count != 1
        return;
    };

    window.title = settings.player.product_name.clone();
}
