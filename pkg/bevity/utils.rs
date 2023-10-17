use std::path::{Path, PathBuf};

pub fn get_assets_dir() -> PathBuf {
    if let Some(path) = std::env::var_os("UNITY_ASSETS_PATH") {
        return Path::new(&path).into();
    } else if let Some(path) = std::env::var_os("CARGO_MANIFEST_DIR") {
        return Path::new(&path).join("../Assets");
    } else {
        panic!("UNABLE TO FIND PATH TO UNITY ASSETS")
    }
}

#[macro_export]
macro_rules! BEVITY_CONST {
    ( $x: ident ) => {
        const $x: &str = stringify!($x);
    };
}
