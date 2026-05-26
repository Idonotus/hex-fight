use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf, absolute},
};

use bevy::{
    ecs::resource::Resource,
    image::Image,
    log::{debug, warn},
    platform::collections::HashMap,
    render::render_resource::TextureFormat,
};
use json::JsonValue;

use manaengine::rendering::assetinterface::{AssetExpectations, ExpectedAssetRef};

fn str_to_textureformat(format: &str) -> TextureFormat {
    match format {
        "RGBAU8" => TextureFormat::Rgba8Uint,
        _ => panic!(),
    }
}

pub fn path_to_abs(path: PathBuf, rel: Option<PathBuf>) -> PathBuf {
    dbg!(path.clone(), rel.clone());
    match rel {
        None => absolute(path).unwrap(),
        Some(p) => absolute(path.join(p)).unwrap(),
    }
}

fn is_asset_path_safe(collection_base: PathBuf, asset_path: &Path) -> bool {
    true //TODO: actual path checking
}

#[derive(Debug, Clone)]
pub(super) enum AssetIndexItem {
    Texture {
        location: PathBuf,
        overrides: Option<DescriptorOverride>,
    },
    Atlas {
        location: PathBuf,
        overrides: Option<DescriptorOverride>,
        dimensions: [u32; 2],
        tile_size: [u32; 2],
        children: Vec<Option<String>>,
    },
    AtlasMember {
        parent: String,
        index: usize,
    },
}

#[derive(Clone, Debug)]
pub(super) struct DescriptorOverride {
    pub format: Option<TextureFormat>,
}

impl DescriptorOverride {
    pub fn apply_override(self, image: &mut Image) {
        if let Some(format) = self.format {
            image.texture_descriptor.format = format;
        }
    }
}

pub trait AssetIndex {
    fn check_index(&self, assets: &[ExpectedAssetRef]) -> bool;
    fn index_asset_collection(
        &mut self,
        root: PathBuf,
        collection: PathBuf,
    ) -> Result<(), &'static str>;
}

#[derive(Resource, Debug, Clone)]
pub struct AssetPackIndex {
    pub(super) index: HashMap<String, AssetIndexItem>,
}

impl AssetPackIndex {
    pub fn new() -> Self {
        AssetPackIndex {
            index: HashMap::new(),
        }
    }
}

impl AssetIndex for AssetPackIndex {
    fn check_index(&self, assets: &[ExpectedAssetRef]) -> bool {
        for expectation in assets.iter() {
            let Some(asset) = self.index.get(&expectation.name) else {
                return false;
            };
            if !asset.meets_expectation(&expectation.expectations) {
                return false;
            }
        }
        return true;
    }

    fn index_asset_collection(
        &mut self,
        root: PathBuf,
        collection: PathBuf,
    ) -> Result<(), &'static str> {
        let p = path_to_abs(root, Some(collection));
        dbg!(p.clone());
        let Ok(mut file) = File::open(&p) else {
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

        let collection_base = p.parent().unwrap();

        for obj in objarr {
            let name = match obj["name"].as_str() {
                Some(n) => n.to_owned(),
                None => {
                    warn!("Unable to index unnamed asset in bundle {collection_base:?}");
                    continue;
                }
            };

            let Some(item): Option<AssetIndexItem> =
                <AssetIndexItem as PartialFrom<JsonValue>>::from(obj)
            else {
                warn!("Cannot parse asset: {name}");
                continue;
            };
            let Ok(item) = absolute_asset_location(collection_base, item) else {
                warn!("Asset {name} is unsafe to access");
                continue;
            };
            self.insert_item_into_index(name, item);
        }

        return Ok(());
    }
}

impl AssetPackIndex {
    fn insert_item_into_index(&mut self, name: String, item: AssetIndexItem) -> () {
        if let Some(prev_item) = self.index.remove(&name) {
            debug!("Overwriting {name}");
            self.prepare_for_override(prev_item)
        }

        match item {
            AssetIndexItem::Atlas {
                location: _,
                overrides: _,
                dimensions: _,
                tile_size: _,
                ref children,
            } => {
                for (idx, c) in children.iter().enumerate() {
                    if let Some(cname) = c {
                        self.insert_item_into_index(
                            cname.clone(),
                            AssetIndexItem::AtlasMember {
                                parent: name.clone(),
                                index: idx,
                            },
                        );
                    }
                }
            }
            _ => {}
        }

        self.index.insert(name, item);
    }

    fn prepare_for_override(&mut self, prev_item: AssetIndexItem) {
        match prev_item {
            AssetIndexItem::Atlas {
                location: _,
                overrides: _,
                dimensions: _,
                tile_size: _,
                children,
            } => {
                children.iter().filter_map(|c| c.as_ref()).for_each(|c| {
                    self.index.remove(c);
                });
            }
            AssetIndexItem::AtlasMember { parent, index } => {
                let Some(&mut AssetIndexItem::Atlas {
                    location: _,
                    overrides: _,
                    dimensions: _,
                    tile_size: _,
                    ref mut children,
                }) = self.index.get_mut(&parent)
                else {
                    panic!()
                };
                children[index] = None;
            }
            _ => {}
        }
    }
}

fn absolute_asset_location(
    collection_base: &Path,
    mut asset: AssetIndexItem,
) -> Result<AssetIndexItem, ()> {
    if !is_asset_path_safe(collection_base.to_owned(), asset.path().unwrap()) {
        return Err(());
    }
    asset.set_path(path_to_abs(
        collection_base.to_owned(),
        Some(asset.path().unwrap().to_owned()),
    ));
    return Ok(asset);
}

impl AssetIndexItem {
    fn texture_from(value: JsonValue) -> Option<Self> {
        return Some(AssetIndexItem::Texture {
            location: value["path"].as_str()?.into(),
            overrides: <DescriptorOverride as PartialFrom<&JsonValue>>::from(&value["overrides"]),
        });
    }
    fn atlas_from(value: JsonValue) -> Option<Self> {
        let mut children: Vec<Option<String>> = Vec::new();
        let JsonValue::Array(ref v) = value["names"] else {
            return None;
        };

        for c in v.iter() {
            if c.is_null() {
                children.push(None);
            }
            children.push(Some(c.as_str()?.to_owned()));
        }
        return Some(AssetIndexItem::Atlas {
            location: value["path"].as_str()?.into(),
            overrides: <DescriptorOverride as PartialFrom<&JsonValue>>::from(&value["overrides"]),
            dimensions: [
                value["dimensions"][0].as_u32()?,
                value["dimensions"][1].as_u32()?,
            ],
            tile_size: [value["tile"][0].as_u32()?, value["tile"][1].as_u32()?],
            children,
        });
    }

    fn path(&self) -> Option<&Path> {
        match self {
            AssetIndexItem::Texture {
                location,
                overrides: _,
            } => Some(&location),
            AssetIndexItem::Atlas {
                location,
                overrides: _,
                dimensions: _,
                tile_size: _,
                children: _,
            } => Some(&location),
            AssetIndexItem::AtlasMember {
                parent: _,
                index: _,
            } => None,
        }
    }

    fn set_path(&mut self, path: PathBuf) -> () {
        match self {
            AssetIndexItem::Texture {
                location,
                overrides: _,
            } => *location = path,
            AssetIndexItem::Atlas {
                location,
                overrides: _,
                dimensions: _,
                tile_size: _,
                children: _,
            } => *location = path,
            _ => panic!(),
        }
    }

    fn possible_expectations(&self) -> Vec<AssetExpectations> {
        match self {
            &AssetIndexItem::Texture {
                location: _,
                overrides: _,
            } => vec![AssetExpectations::ExistsAsImage, AssetExpectations::Texture],
            &AssetIndexItem::AtlasMember {
                parent: _,
                index: _,
            } => vec![
                AssetExpectations::ExistsAsImage,
                AssetExpectations::AtlasChild,
            ],
            &AssetIndexItem::Atlas {
                location: _,
                overrides: _,
                dimensions,
                tile_size: _,
                children: _,
            } => vec![
                AssetExpectations::ExistsAsImage,
                AssetExpectations::Atlas {
                    size: dimensions.clone(),
                },
            ],
        }
    }

    fn meets_expectation(&self, expectation: &AssetExpectations) -> bool {
        self.possible_expectations()
            .iter()
            .any(|p| p == expectation)
    }
}

trait PartialFrom<T>: Sized {
    fn from(value: T) -> Option<Self>;
}

impl PartialFrom<&JsonValue> for DescriptorOverride {
    fn from(value: &JsonValue) -> Option<Self> {
        if value.is_null() {
            return None;
        }
        let overrides = DescriptorOverride {
            format: value["format"].as_str().map(str_to_textureformat),
        };
        return Some(overrides);
    }
}

impl PartialFrom<JsonValue> for AssetIndexItem {
    fn from(value: JsonValue) -> Option<Self> {
        let Some(asset_type) = value["type"].as_str() else {
            return None;
        };

        match asset_type {
            "texture" => Self::texture_from(value),
            "atlas" => Self::atlas_from(value),
            _ => None,
        }
    }
}
