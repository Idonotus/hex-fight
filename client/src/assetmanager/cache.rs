use crate::assetmanager::palettes::BasePalette;

use bevy::{platform::collections::HashMap, prelude::*};
use manaengine::rendering::assetinterface::{Asset as GameAsset, AssetReference, PaletteAllocator};

#[derive(Debug)]
pub(super) enum AssetCacheItem {
    Texture {
        image: Handle<Image>,
    },
    Atlas {
        image: Handle<Image>,
        layout: Handle<TextureAtlasLayout>,
    },
    AtlasMember {
        parent: String,
        index: usize,
    },
}

pub trait AssetContainer {
    fn fetch_assets(
        &self,
        references: Vec<AssetReference>,
    ) -> Vec<Result<GameAsset, ReferenceError>>;
}

#[derive(Debug)]
pub enum ReferenceError {
    NotFound(String),
    Mismatch(String, &'static str),
}

#[derive(Resource)]
pub struct AssetCache {
    pub(super) assets: HashMap<String, AssetCacheItem>,
    pub(super) palette: BasePalette,
}

impl AssetContainer for AssetCache {
    fn fetch_assets(
        &self,
        references: Vec<AssetReference>,
    ) -> Vec<Result<GameAsset, ReferenceError>> {
        return references
            .into_iter()
            .map(|aref| match aref {
                AssetReference::Texture(name) => self.fetch_texture(name),
                AssetReference::Palette(r) => Ok(GameAsset::Palette(
                    self.palette.img.clone(),
                    r,
                    self.palette.allocator.get_size(),
                )),
                AssetReference::AtlasIndex(aname, index) => {
                    let Some(AssetCacheItem::Atlas { image, layout }) = self.assets.get(&aname)
                    else {
                        panic!()
                    };
                    Ok(GameAsset::AtlasTexture(
                        image.clone(),
                        layout.clone(),
                        index,
                    ))
                }
            })
            .collect();
    }
}

impl AssetCache {
    pub fn new(palette: BasePalette) -> Self {
        return Self {
            assets: HashMap::new(),
            palette,
        };
    }

    fn fetch_texture(&self, texture_name: String) -> Result<GameAsset, ReferenceError> {
        let Some(asset) = self.assets.get(&texture_name) else {
            return Err(ReferenceError::NotFound(texture_name));
        };

        match asset {
            &AssetCacheItem::Texture { ref image } => Ok(GameAsset::Texture(image.clone())),
            &AssetCacheItem::Atlas {
                ref image,
                layout: _,
            } => Ok(GameAsset::Texture(image.clone())),
            &AssetCacheItem::AtlasMember { ref parent, index } => {
                let Some(AssetCacheItem::Atlas { image, layout }) = self.assets.get(parent) else {
                    return Err(ReferenceError::Mismatch(parent.clone(), "atlas"));
                };
                Ok(GameAsset::AtlasTexture(
                    image.clone(),
                    layout.clone(),
                    index,
                ))
            }
            // Future proofing
            _ => Err(ReferenceError::Mismatch(texture_name, "texture based")),
        }
    }
}
