use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context, Result};
use bevity_primitives::UnityMaterial;
use bevity_yaml::parse_unity_yaml;
use serde::{Deserialize, Serialize};

pub fn read_materials(base: &Path, config_file: &Path) -> Result<HashMap<String, UnityMaterial>> {
    let materials =
        std::fs::read_to_string(config_file).context("failed to read materials.json")?;
    let materials: HashMap<String, String> =
        serde_json::from_str(&materials).context("failed to parse materials.json")?;

    materials
        .into_iter()
        .try_fold(HashMap::new(), |mut acc, (k, v)| {
            let path = base.join("..").join(v);
            let contents = std::fs::read_to_string(path)?;
            let mat = read_single_material(&contents).context("failed to read single material")?;

            acc.insert(k, mat);

            Ok(acc)
        })
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
enum MaterialContainer {
    Material(UnityMaterial),
    #[serde(other)]
    DontCare,
}

fn read_single_material(contents: &str) -> Result<UnityMaterial> {
    let map = parse_unity_yaml(contents)?;

    let (_, output) = map.into_iter().next().context("0 items in material file")?;

    let MaterialContainer::Material(mat) = output else {
        bail!("invalid material file");
    };

    Ok(mat)
}
