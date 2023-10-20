use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    ecs::system::EntityCommands,
    prelude::*,
    render::{
        camera::ScalingMode,
        render_resource::{TextureViewDescriptor, TextureViewDimension},
    },
};
use serde::{Deserialize, Serialize};

use crate::UnityColor;

#[derive(Serialize, Deserialize, Debug, Default, Copy, Clone)]
pub struct UnityCamera {
    #[serde(alias = "m_BackGroundColor")]
    pub background_color: UnityColor,

    #[serde(alias = "near clip plane")]
    pub near_clip_plane: f32,

    #[serde(alias = "far clip plane")]
    pub far_clip_plane: f32,

    #[serde(alias = "field of view")]
    pub fov: f32,

    pub orthographic: i32,

    #[serde(alias = "orthographic size")]
    pub orthographic_size: f32,
}

#[derive(Component, Debug, Clone)]
pub struct UnityCameraSkyboxCubemap {
    skybox: Option<Handle<Image>>,
    skybox_loaded: bool,
}

impl UnityCamera {
    pub fn add_camera_bundle(
        &self,
        transform: Transform,
        skybox: Option<&Handle<Image>>,
        commands: &mut EntityCommands,
    ) {
        let projection = if self.orthographic == 1 {
            Projection::Orthographic(OrthographicProjection {
                near: self.near_clip_plane,
                far: self.far_clip_plane,
                scaling_mode: ScalingMode::WindowSize(self.orthographic_size * 3.0),
                ..default()
            })
        } else {
            Projection::Perspective(PerspectiveProjection {
                fov: self.fov.to_radians(),
                aspect_ratio: 1.0,
                near: self.near_clip_plane,
                far: self.far_clip_plane,
            })
        };

        commands.insert(Camera3dBundle {
            transform,
            projection,
            ..default()
        });

        if skybox.is_some() {
            commands.insert(UnityCameraSkyboxCubemap {
                skybox: skybox.cloned(),
                skybox_loaded: false,
            });
        }
    }
}

pub fn load_camera_skybox_system(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut unity_cameras: Query<(Entity, &mut UnityCameraSkyboxCubemap)>,
    mut commands: Commands,
) {
    for (entity, mut meta) in &mut unity_cameras {
        let Some(skybox) = &meta.skybox else {
            meta.skybox_loaded = true; // no skybox, shut this down
            continue;
        };

        if asset_server.get_load_state(skybox) != LoadState::Loaded {
            continue; //not ready yet
        }

        let Some(image) = images.get_mut(skybox) else {
            continue;
        };

        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(
                image.texture_descriptor.size.height / image.texture_descriptor.size.width,
            );
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        let mut commands = commands.entity(entity);
        commands.insert(Skybox(skybox.clone()));
        commands.insert(EnvironmentMapLight {
            diffuse_map: skybox.clone(),
            specular_map: skybox.clone(),
        });

        commands.remove::<UnityCameraSkyboxCubemap>();
    }
}
