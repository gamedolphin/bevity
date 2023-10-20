use std::marker::PhantomData;

use bevity_scene::{MonoBehaviour, SceneResource, UnityScene, BEVITY_CONST};
use bevy::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

mod stdin;
mod stdout;

pub use stdout::UnityChangeMap;

BEVITY_CONST!(ENABLE_BEVITY_EDITOR);
BEVITY_CONST!(BEVITY_EDITOR_SCENE);
BEVITY_CONST!(BEVITY_EDITOR_SCENE_GUID);

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

        let Some(scene_guid) = std::env::var_os(BEVITY_EDITOR_SCENE_GUID) else {
            return;
        };

        let Some(scene_path) = get_scene_path() else {
            tracing::error!("expected a scene path in bevy editor plugin, found None");
            return;
        };

        let scene =
            match bevity_scene::parse_scene_file::<T>(&scene_guid.to_string_lossy(), &scene_path) {
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

        stdin::setup_stdin::<T>(app);
        stdout::setup_stdout::<T>(app);

        println!("setup editor watch!");
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
