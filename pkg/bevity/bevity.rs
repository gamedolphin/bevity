use std::marker::PhantomData;

use bevy::prelude::*;
use editor::EditorPlugin;
use resources::ResourcesPlugin;
use scenes::ScenePlugin;
use serde::Serialize;
use settings::SettingsPlugin;

mod editor;
mod resources;
mod scenes;
mod settings;
mod stdin;
mod stdout;
mod utils;

pub use scenes::MonoBehaviour;
pub use stdin::UnityChangeObject;
pub use stdout::UnityChangeMap;

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
            + MonoBehaviour,
    > Plugin for BevityPlugin<T>
{
    fn build(&self, app: &mut App) {
        app.add_plugins(ScenePlugin::<T>::default());
        app.add_plugins(EditorPlugin::<T>::default());
        app.add_plugins((SettingsPlugin, ResourcesPlugin));
    }
}
