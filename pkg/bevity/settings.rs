use bevity_settings::PlayerSettings;
use bevy::prelude::*;
use std::path::Path;

#[derive(Default)]
pub struct SettingsPlugin;

#[derive(Resource, Debug, Default)]
pub struct UnitySettings {
    pub player: PlayerSettings,
}

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        let path = crate::utils::get_assets_dir();

        let path = Path::new(&path).join("..");
        let Ok(player) = bevity_settings::parse_project_settings_file(&path) else {
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
