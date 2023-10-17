use bevy::{ecs::system::EntityCommands, prelude::*};
use serde::{Deserialize, Serialize};

use crate::FileReference;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityMeshFilter {
    #[serde(alias = "m_Mesh")]
    pub mesh: FileReference,
}

impl UnityMeshFilter {
    pub fn add_mesh_filter_meta(&self, object_id: u64, commands: &mut EntityCommands) {
        commands.insert(UnityMeshFilterMeta {
            object_id,
            mesh: self.mesh.clone(),
        });

        commands.insert(UnityMeshDirty);
    }
}

#[derive(Component, Debug)]
pub struct UnityMeshFilterMeta {
    pub object_id: u64,
    pub mesh: FileReference,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UnityMeshRenderer {
    #[serde(alias = "m_Materials")]
    pub materials: Vec<FileReference>,
}

impl UnityMeshRenderer {
    pub fn add_mesh_renderer_meta(&self, object_id: u64, commands: &mut EntityCommands) {
        commands.insert(UnityMeshRendererMeta {
            object_id,
            materials: self.materials.clone(),
        });

        commands.insert(UnityMeshDirty);
    }
}

#[derive(Component, Debug)]
pub struct UnityMeshRendererMeta {
    pub object_id: u64,
    pub materials: Vec<FileReference>,
}

#[derive(Component, Debug)]
pub struct UnityMeshDirty;
