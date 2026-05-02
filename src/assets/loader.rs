use std::{
    fs::File,
    io::Read,
    path::{PathBuf, absolute},
};

use json::JsonValue;

use crate::assets::cache::AssetCache;

pub fn path_to_abs(path: &str, rel: Option<PathBuf>) -> PathBuf {
    if !path.starts_with(".") {
        return PathBuf::from(path);
    }
    match rel {
        None => absolute(path).unwrap(),
        Some(p) => absolute(p.join(path)).unwrap(),
    }
}

pub fn load_pack_index(path: PathBuf, cache: &mut AssetCache) -> Result<(), &'static str> {
    let p = path_to_abs(&("./assets/".to_owned() + path.to_str().unwrap()), None);
    let Ok(mut file) = File::open(p) else {
        return Err("Error opening file");
    };
    let mut buffer = String::new();
    let Ok(_) = file.read_to_string(&mut buffer) else {
        return Err("Error reading file");
    };

    let Ok(j) = json::parse(&buffer) else {
        return Err("Error parsing file");
    };

    let JsonValue::Array(objarr) = j else {
        return Err("Pack is not an array");
    };

    let index_base = path.parent().unwrap();

    for obj in objarr {
        cache.insert_into_index(index_base.to_owned(), obj);
    }

    return Ok(());
}
