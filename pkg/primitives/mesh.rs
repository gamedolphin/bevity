use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

use crate::FileReference;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityMeshFilter {
    #[serde(alias = "m_Mesh")]
    pub mesh: FileReference,
}

impl UnityMeshFilter {
    pub fn add_mesh_filter_meta(&self, commands: &mut EntityCommands) {
        commands.insert(UnityMeshFilterExtra {
            mesh: self.mesh.clone(),
        });
        commands.insert(UnityMeshRequiresLoad);
    }
}

#[derive(Component, Debug)]
pub struct UnityMeshFilterExtra {
    pub mesh: FileReference,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityMeshRenderer {
    #[serde(alias = "m_Materials")]
    pub materials: Vec<FileReference>,
}

impl UnityMeshRenderer {
    pub fn add_mesh_renderer_meta(&self, commands: &mut EntityCommands) {
        commands.insert(UnityMeshRendererExtra {
            materials: self.materials.clone(),
        });
        commands.insert(UnityMeshRequiresLoad);
    }
}

#[derive(Component, Debug)]
pub struct UnityMeshRendererExtra {
    pub materials: Vec<FileReference>,
}

#[derive(Component, Debug)]
pub struct UnityMeshRequiresLoad;
