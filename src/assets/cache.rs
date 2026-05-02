use std::{
    any::TypeId,
    mem::swap,
    path::{Path, PathBuf},
};

use bevy::{
    asset::LoadState,
    platform::collections::HashMap,
    prelude::*,
    render::render_resource::TextureFormat,
    sprite_render::{Material2d, Material2dPlugin},
};
use json::JsonValue;

use super::palette::{BasePalette, PaletteAtlas, PaletteReference};

pub enum AssetReference {
    Texture(String),
    Palette(PaletteReference),
    AtlasTexture(String),
    AtlasIndex(String, usize),
}

#[derive(Clone)]
pub struct DescriptorOverride {
    pub format: Option<TextureFormat>,
}

impl DescriptorOverride {
    fn apply_override(self, image: &mut Image) {
        if let Some(format) = self.format {
            image.texture_descriptor.format = format;
        }
    }
}

#[derive(Debug)]
pub enum Asset {
    Texture(Handle<Image>),
    Palette(Handle<Image>, PaletteReference, PaletteReference),
    AtlasTexture(Handle<Image>, Handle<TextureAtlasLayout>, usize),
}

enum AssetIndexItem {
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

#[derive(Debug)]
enum AssetCacheItem {
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

fn str_to_textureformat(format: &str) -> TextureFormat {
    match format {
        "RGBAU8" => TextureFormat::Rgba8Uint,
        _ => panic!(),
    }
}

#[derive(Resource)]
pub struct AssetCache {
    pub index: HashMap<String, AssetIndexItem>,
    assets: HashMap<String, AssetCacheItem>,
    pub palette: BasePalette,
    image_queue: Vec<(Handle<Image>, DescriptorOverride)>,
}

pub struct AssetInterface<'a> {
    pub server: ResMut<'a, AssetServer>,
    pub layouts: ResMut<'a, Assets<TextureAtlasLayout>>,
}

impl AssetCache {
    pub fn new(palette: BasePalette) -> Self {
        return Self {
            index: HashMap::new(),
            assets: HashMap::new(),
            image_queue: Vec::new(),
            palette,
        };
    }

    pub fn load(&mut self, mut loader: AssetInterface, assets: Vec<String>) {
        for a in assets {
            if self.assets.contains_key(&a) {
                continue;
            }

            if !self.index.contains_key(&a) {
                println!("No asset called {} found", a);
                panic!();
            }

            let i = self.index.get(&a).unwrap();
            match i {
                AssetIndexItem::Texture {
                    location,
                    overrides,
                } => {
                    let image = loader.server.load(location.clone());
                    if let Some(o) = overrides {
                        self.image_queue.push((image.clone(), o.clone()));
                    }
                    self.assets.insert(a, AssetCacheItem::Texture { image });
                }
                AssetIndexItem::Atlas {
                    overrides,
                    location,
                    dimensions,
                    tile_size,
                    children,
                } => {
                    let (image, layout) = AssetCache::load_atlas_materials(
                        &mut self.image_queue,
                        &mut loader,
                        location,
                        dimensions,
                        tile_size,
                        overrides,
                    );
                    AssetCache::load_atlas(&mut self.assets, &a, image, layout, children);
                }
                AssetIndexItem::AtlasMember { parent, index: _ } => {
                    let Some(AssetIndexItem::Atlas {
                        overrides,
                        location,
                        dimensions,
                        tile_size,
                        children,
                    }) = self.index.get(parent)
                    else {
                        println!("Atlas item '{}' is referenced incorrectly", a);
                        panic!()
                    };
                    let (image, layout) = AssetCache::load_atlas_materials(
                        &mut self.image_queue,
                        &mut loader,
                        location,
                        dimensions,
                        tile_size,
                        overrides,
                    );
                    AssetCache::load_atlas(&mut self.assets, parent, image, layout, children);
                }
            }
        }
    }

    fn load_atlas_materials(
        image_queue: &mut Vec<(Handle<Image>, DescriptorOverride)>,
        loader: &mut AssetInterface,
        location: &PathBuf,
        dimensions: &[u32],
        tile_size: &[u32],
        overrides: &Option<DescriptorOverride>,
    ) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
        let image = loader.server.load(location.clone());
        if let Some(o) = overrides {
            image_queue.push((image.clone(), o.clone()));
        }
        let layout = loader.layouts.add(TextureAtlasLayout::from_grid(
            UVec2 {
                x: tile_size[0],
                y: tile_size[1],
            },
            dimensions[0],
            dimensions[1],
            None,
            None,
        ));
        return (image, layout);
    }

    fn load_atlas(
        assets: &mut HashMap<String, AssetCacheItem>,
        name: &String,
        image: Handle<Image>,
        layout: Handle<TextureAtlasLayout>,
        children: &Vec<Option<String>>,
    ) {
        assets.insert(name.clone(), AssetCacheItem::Atlas { image, layout });
        for (idx, child) in children.iter().enumerate() {
            let Some(c) = child else {
                continue;
            };
            assets.insert(
                c.clone(),
                AssetCacheItem::AtlasMember {
                    parent: name.clone(),
                    index: idx,
                },
            );
        }
    }

    pub fn check_index(&self, assets: &Vec<String>) -> bool {
        !assets.iter().any(|n| !self.index.contains_key(n))
    }

    pub fn insert_into_index(&mut self, base: PathBuf, obj: JsonValue) -> () {
        let n = obj["name"].as_str().unwrap().to_owned();
        println!("Added '{}' to index", n);
        let Some(t) = obj["type"].as_str() else {
            return;
        };
        let Some(p) = obj["path"].as_str() else {
            return;
        };
        let location = base.join(p);
        println!("{:?}\n{:?}", base, location);
        match t {
            "texture" => {
                let overrides = index_overrides(obj["overrides"].clone());
                self.index.insert(
                    n,
                    AssetIndexItem::Texture {
                        location,
                        overrides,
                    },
                );
            }
            "atlas" => {
                let overrides = index_overrides(obj["overrides"].clone());
                let mut children = Vec::new();
                let JsonValue::Array(ref v) = obj["names"] else {
                    panic!("child names");
                    return;
                };
                for (idx, c) in v.iter().enumerate() {
                    let cname = c.as_str().unwrap().to_owned();
                    self.index.insert(
                        cname.clone(),
                        AssetIndexItem::AtlasMember {
                            parent: n.clone(),
                            index: idx,
                        },
                    );
                    children.push(Some(cname));
                }
                self.index.insert(
                    n.to_owned(),
                    AssetIndexItem::Atlas {
                        location,
                        overrides,
                        dimensions: [
                            obj["dimensions"][0].as_u32().unwrap(),
                            obj["dimensions"][1].as_u32().unwrap(),
                        ],
                        tile_size: [
                            obj["tile"][0].as_u32().unwrap(),
                            obj["tile"][1].as_u32().unwrap(),
                        ],
                        children,
                    },
                );
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn get_assets(&self, ref_list: Vec<AssetReference>) -> Vec<Asset> {
        return ref_list
            .into_iter()
            .map(|aref| match aref {
                AssetReference::Texture(name) => {
                    let Some(AssetCacheItem::Texture { image }) = self.assets.get(&name) else {
                        self.mismatch_panic(&name, "texture")
                    };
                    Asset::Texture(image.clone())
                }
                AssetReference::Palette(r) => {
                    Asset::Palette(self.palette.get_image(), r, self.palette.get_size())
                }
                AssetReference::AtlasTexture(segname) => {
                    let Some(AssetCacheItem::AtlasMember { parent, index }) =
                        self.assets.get(&segname)
                    else {
                        self.mismatch_panic(&segname, "atlas child")
                    };
                    let Some(AssetCacheItem::Atlas { image, layout }) = self.assets.get(parent)
                    else {
                        panic!()
                    };
                    Asset::AtlasTexture(image.clone(), layout.clone(), *index)
                }
                AssetReference::AtlasIndex(aname, index) => {
                    let Some(AssetCacheItem::Atlas { image, layout }) = self.assets.get(&aname)
                    else {
                        panic!()
                    };
                    Asset::AtlasTexture(image.clone(), layout.clone(), index)
                }
            })
            .collect();
    }

    pub fn mismatch_panic<T>(&self, name: &str, asset_type: &str) -> T {
        panic!(
            "Asset mismatch! '{}' expected {} but ended up with {:?}",
            name,
            asset_type,
            self.assets.get(name)
        )
    }

    pub fn loading(&mut self, server: Res<AssetServer>, mut images: ResMut<Assets<Image>>) -> bool {
        let mut newqueue = Vec::new();
        swap(&mut newqueue, &mut self.image_queue);
        for (h, o) in newqueue.drain(..) {
            match server.get_load_state(&h).unwrap() {
                LoadState::NotLoaded | LoadState::Loading => {
                    self.image_queue.push((h, o));
                }
                LoadState::Failed(_) => {}
                LoadState::Loaded => {
                    o.apply_override(images.get_mut(&h).unwrap());
                }
            }
        }

        return self.image_queue.len() == 0;
    }
}

fn index_overrides(overrides: JsonValue) -> Option<DescriptorOverride> {
    if overrides.is_null() {
        None
    } else {
        Some(DescriptorOverride {
            format: overrides["format"].as_str().map(str_to_textureformat),
        })
    }
}

pub struct MaterialCache<'a> {
    app: &'a mut App,
    pub materials: Vec<TypeId>,
}

impl<'a> MaterialCache<'a> {
    pub fn new(app: &'a mut App) -> Self {
        Self {
            app,
            materials: Vec::new(),
        }
    }

    pub fn add_mat<T>(&mut self) -> ()
    where
        T: Material2d,
        Material2dPlugin<T>: Plugin,
    {
        let t = TypeId::of::<T>();
        if self.materials.iter().any(|r| *r == t) {
            return;
        }
        self.app.add_plugins((Material2dPlugin::<T>::default(),));
    }
}
