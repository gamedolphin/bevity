use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bevity_materials::UnityMaterial;
use bevy::prelude::*;

use crate::utils::get_assets_dir;

#[derive(Default)]
pub struct ResourcesPlugin;

#[derive(Resource, Debug, Default)]
pub struct UnityResource {
    pub base_path: PathBuf,
    pub textures: HashMap<String, Handle<Image>>,
    pub standard_materials: HashMap<String, Handle<StandardMaterial>>,

    pub meshes: HashMap<String, Handle<Mesh>>,

    pub materials_map: HashMap<String, UnityMaterial>,
    pub textures_map: HashMap<String, String>,
}

impl Plugin for ResourcesPlugin {
    fn build(&self, app: &mut App) {
        let path = get_assets_dir();
        let path = Path::new(&path);
        let materials_json = path.join("materials.json");

        let materials = match bevity_materials::read_materials(path, &materials_json) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("failed to read materials: {:?}", e);
                return;
            }
        };

        let textures_json = path.join("textures.json");
        let Ok(textures_map) = read_texture_map(&textures_json) else {
            tracing::error!("failed to parse texture json");
            return;
        };

        app.insert_resource(UnityResource {
            base_path: path.into(),
            materials_map: materials,
            textures_map,
            ..default()
        })
        .add_systems(
            Startup,
            (
                load_textures_system,
                load_materials.after(load_textures_system),
            ),
        );
    }
}

fn read_texture_map(path: &Path) -> Result<HashMap<String, String>> {
    let file = std::fs::read_to_string(path)?;
    let texture_pathmap: HashMap<String, String> = serde_json::from_str(&file)?;

    Ok(texture_pathmap)
}

fn load_textures_system(
    asset_server: Res<AssetServer>,
    mut unity_resources: ResMut<UnityResource>,
) {
    let Ok(textures) = load_textures(
        &unity_resources.base_path,
        &unity_resources.textures_map,
        asset_server,
    ) else {
        tracing::error!("failed to load textures from the unity side");
        return;
    };

    unity_resources.textures = textures;
}

fn load_textures(
    base: &Path,
    texture_pathmap: &HashMap<String, String>,
    asset_server: Res<AssetServer>,
) -> Result<HashMap<String, Handle<Image>>> {
    texture_pathmap
        .iter()
        .try_fold(HashMap::new(), |mut acc, (k, v)| {
            let path = base.join("..").join(v);
            let handle = asset_server.load(path);

            acc.insert(k.clone(), handle);

            Ok(acc)
        })
}

fn load_materials(
    mut unity_res: ResMut<UnityResource>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    unity_res.standard_materials = unity_res
        .materials_map
        .iter()
        .filter_map(|(guid, f)| {
            let standard_material = f.get_standard_material()?;
            Some((guid, standard_material))
        })
        .fold(HashMap::new(), |mut acc, (guid, mat)| {
            let mat = materials.add(mat);
            acc.insert(guid.clone(), mat);

            acc
        });
}
