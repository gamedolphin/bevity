use std::{
    collections::HashMap,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use anyhow::Result;
use bevity_primitives::UnityMaterial;
use bevy::{gltf::Gltf, prelude::*};

use crate::UnityScene;

#[derive(Default)]
pub struct ResourcesPlugin<T>(PhantomData<T>);

#[derive(Resource, Default)]
pub struct UnityResource<T: Default> {
    pub base_path: PathBuf,
    pub textures: HashMap<String, Handle<Image>>,
    pub standard_materials: HashMap<String, Handle<StandardMaterial>>,
    pub gltfs: HashMap<String, Handle<Gltf>>,
    pub models: HashMap<String, Handle<Scene>>,

    pub meshes: HashMap<String, Handle<Mesh>>,

    pub materials_map: HashMap<String, UnityMaterial>,
    pub textures_map: HashMap<String, String>,
    pub prefabs: HashMap<String, UnityScene<T>>,

    pub all_map: HashMap<String, String>,
}

impl<T: Sync + Send + 'static + Default> Plugin for ResourcesPlugin<T> {
    fn build(&self, app: &mut App) {
        let path = std::env::current_dir().unwrap();
        let path = Path::new(&path);
        let materials_json = path.join("materials.json");

        let materials = match crate::read_materials(path, &materials_json) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("failed to read materials: {:?}", e);
                return;
            }
        };

        let textures_json = path.join("textures.json");
        let Ok(textures_map) = read_guid_path_map(&textures_json) else {
            tracing::error!("failed to parse texture json");
            return;
        };

        let all_json = path.join("all.json");
        let Ok(all_map) = read_guid_path_map(&all_json) else {
            tracing::error!("failed to parse all json");
            return;
        };

        app.insert_resource(UnityResource::<T> {
            base_path: path.into(),
            materials_map: materials,
            textures_map,
            all_map,
            ..default()
        })
        .add_systems(
            Startup,
            (
                load_textures_system::<T>,
                load_materials::<T>.after(load_textures_system::<T>),
            ),
        );
    }
}

fn read_guid_path_map(path: &Path) -> Result<HashMap<String, String>> {
    let file = std::fs::read_to_string(path)?;
    let texture_pathmap: HashMap<String, String> = serde_json::from_str(&file)?;

    Ok(texture_pathmap)
}

fn load_textures_system<T: Default + Sync + Send + 'static>(
    asset_server: Res<AssetServer>,
    mut unity_resources: ResMut<UnityResource<T>>,
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

fn load_materials<T: Sync + Send + 'static + Default>(
    mut unity_res: ResMut<UnityResource<T>>,
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
