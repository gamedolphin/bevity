use std::marker::PhantomData;

use bevity_scene::UnityScene;
use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    scenes::SceneResource, stdin::setup_stdin, stdout::setup_stdout, MonoBehaviour, BEVITY_CONST,
};

BEVITY_CONST!(ENABLE_BEVITY_EDITOR);
BEVITY_CONST!(BEVITY_EDITOR_SCENE);

#[derive(Default)]
pub struct EditorPlugin<T: DeserializeOwned + Clone + Sync + Send + Default + 'static>(
    PhantomData<T>,
);

#[derive(Resource, Default)]
pub struct EditorResource<T> {
    pub current_scene_name: String,
    pub current_scene: UnityScene<T>,
}

impl<T: Clone + Sync + Send + Default + DeserializeOwned + Serialize + 'static + MonoBehaviour>
    Plugin for EditorPlugin<T>
{
    fn build(&self, app: &mut App) {
        if std::env::var_os(ENABLE_BEVITY_EDITOR).is_none() {
            return;
        }

        let Some(scene_path) = get_scene_path() else {
            tracing::error!("expected a scene path in bevy editor plugin, found None");
            return;
        };

        let scene = match bevity_scene::parse_scene_file::<T>(&scene_path) {
            Ok(scene) => scene,
            Err(e) => {
                tracing::error!("failed to parse unity scene: {:?}", e);
                return;
            }
        };

        app.insert_resource(EditorResource {
            current_scene_name: scene_path,
            current_scene: scene,
        })
        .add_systems(Startup, set_initial_scene::<T>);

        setup_stdin::<T>(app);
        setup_stdout::<T>(app);
    }
}

fn set_initial_scene<T: Clone + Sync + Send + 'static>(
    editor: Res<EditorResource<T>>,
    mut scenes: ResMut<SceneResource<T>>,
) {
    scenes.scenes.insert(
        editor.current_scene_name.clone(),
        editor.current_scene.clone(),
    );

    scenes.current = Some(editor.current_scene_name.clone())
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
