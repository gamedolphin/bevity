use bevy::prelude::*;

#[macro_export]
macro_rules! BEVITY_CONST {
    ( $x: ident ) => {
        const $x: &str = stringify!($x);
    };
}

BEVITY_CONST!(ENABLE_BEVITY_EDITOR);
BEVITY_CONST!(BEVITY_EDITOR_SCENE);

#[derive(Default)]
pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var_os(ENABLE_BEVITY_EDITOR).is_none() {
            return;
        }

        let Some(scene_path) = get_scene_path() else {
            tracing::error!("expected a scene path in bevy editor plugin, found None");
            return;
        };

        let scene = match bevity_scene::parse_scene_file(&scene_path) {
            Ok(scene) => scene,
            Err(e) => {
                tracing::error!("failed to parse unity scene: {}", e);
                return;
            }
        };
    }
}

fn get_scene_path() -> Option<String> {
    let Some(scene_path) = std::env::var_os(BEVITY_EDITOR_SCENE) else {
        return None;
    };

    let Some(scene_path) = scene_path.to_str() else {
        return None;
    };

    Some(scene_path.to_string())
}
