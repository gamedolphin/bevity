use std::marker::PhantomData;

use bevity_editor::EditorPlugin;
use bevity_scene::ScenePlugin;
use bevy::prelude::*;
use serde::Serialize;

pub use bevity_builder::build;
pub use bevity_editor::UnityChangeMap;
pub use bevity_editor::BEVITY_EDITOR_SCENE;
pub use bevity_editor::ENABLE_BEVITY_EDITOR;
pub use bevity_generator::exported_component_list;
pub use bevity_scene::MonoBehaviour;
pub use bevity_scene::UnitySceneObject;

#[derive(Default)]
pub struct BevityPlugin<T> {
    marker: PhantomData<T>,
}

impl<
        T: serde::de::DeserializeOwned
            + Serialize
            + Clone
            + Sync
            + Send
            + Default
            + 'static
            + MonoBehaviour
            + Plugin,
    > Plugin for BevityPlugin<T>
{
    fn build(&self, app: &mut App) {
        app.add_plugins(ScenePlugin::<T>::default());
        app.add_plugins(EditorPlugin::<T>::default());
        app.add_plugins(T::default());
        app.add_plugins(bevity_settings::SettingsPlugin);
    }
}
