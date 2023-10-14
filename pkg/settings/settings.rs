use std::path::Path;

use anyhow::{bail, Context, Result};
use bevity_yaml::parse_unity_yaml;

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
