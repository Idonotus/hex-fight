use std::{mem::swap, path::PathBuf};

use crate::assetmanager::{
    cache::{AssetCache, AssetCacheItem, AssetContainer},
    indexer::{AssetIndex, AssetIndexItem, AssetPackIndex, DescriptorOverride},
};
use bevy::{
    asset::LoadState, ecs::system::SystemState, platform::collections::HashMap, prelude::*,
};

pub struct LoadingInterface<'a, I: AssetIndex + Resource, C: AssetContainer + Resource> {
    pub server: ResMut<'a, AssetServer>,
    pub layouts: ResMut<'a, Assets<TextureAtlasLayout>>,
    pub index: Res<'a, I>,
    pub images: ResMut<'a, Assets<Image>>,
    pub container: ResMut<'a, C>,
}

impl<'a, I: AssetIndex + Resource, C: AssetContainer + Resource> LoadingInterface<'a, I, C> {
    fn new(world: &'a mut World) -> LoadingInterface<'a, I, C> {
        let (server, layouts, index, images, container) = SystemState::<(
            ResMut<AssetServer>,
            ResMut<Assets<TextureAtlasLayout>>,
            Res<I>,
            ResMut<Assets<Image>>,
            ResMut<C>,
        )>::new(world)
        .get_mut(world);
        return Self {
            server,
            layouts,
            index,
            images,
            container,
        };
    }
}

pub trait AssetLoader<I: AssetIndex + Resource, C: AssetContainer + Resource> {
    fn loading_step<'a>(&mut self, loader: LoadingInterface<'a, I, C>) -> bool;
    fn load<'a>(&mut self, loader: LoadingInterface<'a, I, C>, assets: Vec<String>);
}

pub trait PrepLoader<'a, T, I: AssetIndex + Resource, C: AssetContainer + Resource> {
    fn prep_vars_from_world(world: &'a mut World) -> (T, LoadingInterface<'a, I, C>);
}

impl<'a, T: AssetLoader<I, C> + Resource, I: AssetIndex + Resource, C: AssetContainer + Resource>
    PrepLoader<'a, ResMut<'a, T>, I, C> for T
{
    fn prep_vars_from_world(world: &'a mut World) -> (ResMut<'a, T>, LoadingInterface<'a, I, C>) {
        let (selfres, server, layouts, index, images, container) = SystemState::<(
            ResMut<T>,
            ResMut<AssetServer>,
            ResMut<Assets<TextureAtlasLayout>>,
            Res<I>,
            ResMut<Assets<Image>>,
            ResMut<C>,
        )>::new(world)
        .get_mut(world);
        return (
            selfres,
            LoadingInterface {
                server,
                layouts,
                index,
                images,
                container,
            },
        );
    }
}

#[derive(Resource)]
pub struct CacheLoader {
    image_queue: Vec<(Handle<Image>, DescriptorOverride)>,
}

impl AssetLoader<AssetPackIndex, AssetCache> for CacheLoader {
    fn load(
        &mut self,
        mut loader: LoadingInterface<'_, AssetPackIndex, AssetCache>,
        assets: Vec<String>,
    ) {
        for a in assets {
            if loader.container.assets.contains_key(&a) {
                continue;
            }

            if !loader.index.index.contains_key(&a) {
                println!("No asset called {} found", a);
                panic!();
            }

            let i = loader.index.index.get(&a).unwrap();
            match i {
                AssetIndexItem::Texture {
                    location,
                    overrides,
                } => {
                    let image = loader.server.load(location.clone());
                    if let Some(o) = overrides {
                        self.image_queue.push((image.clone(), o.clone()));
                    }
                    loader
                        .container
                        .assets
                        .insert(a, AssetCacheItem::Texture { image });
                }
                AssetIndexItem::Atlas {
                    overrides,
                    location,
                    dimensions,
                    tile_size,
                    children,
                } => {
                    let (image, layout) = self.load_atlas_materials(
                        &mut loader.server,
                        &mut loader.layouts,
                        location,
                        dimensions,
                        tile_size,
                        overrides,
                    );
                    CacheLoader::load_atlas(
                        &mut loader.container.assets,
                        &a,
                        image,
                        layout,
                        children,
                    );
                }
                AssetIndexItem::AtlasMember { parent, index: _ } => {
                    let Some(AssetIndexItem::Atlas {
                        overrides,
                        location,
                        dimensions,
                        tile_size,
                        children,
                    }) = loader.index.index.get(parent)
                    else {
                        println!("Atlas item '{}' is referenced incorrectly", a);
                        panic!()
                    };
                    let (image, layout) = self.load_atlas_materials(
                        &mut loader.server,
                        &mut loader.layouts,
                        location,
                        dimensions,
                        tile_size,
                        overrides,
                    );
                    CacheLoader::load_atlas(
                        &mut loader.container.assets,
                        parent,
                        image,
                        layout,
                        children,
                    );
                }
            }
        }
    }

    fn loading_step<'a>(
        &mut self,
        mut loader: LoadingInterface<'a, AssetPackIndex, AssetCache>,
    ) -> bool {
        let mut newqueue = Vec::new();
        swap(&mut newqueue, &mut self.image_queue);
        for (h, o) in newqueue.drain(..) {
            match loader.server.get_load_state(&h).unwrap() {
                LoadState::NotLoaded | LoadState::Loading => {
                    self.image_queue.push((h, o));
                }
                LoadState::Failed(_) => {}
                LoadState::Loaded => {
                    o.apply_override(loader.images.get_mut(&h).unwrap());
                }
            }
        }

        return self.image_queue.len() == 0;
    }
}

impl CacheLoader {
    pub fn new() -> Self {
        Self {
            image_queue: Vec::new(),
        }
    }

    fn load_atlas_materials(
        &mut self,
        server: &mut ResMut<AssetServer>,
        layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        location: &PathBuf,
        dimensions: &[u32],
        tile_size: &[u32],
        overrides: &Option<DescriptorOverride>,
    ) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
        let image = server.load(location.clone());
        if let Some(o) = overrides {
            self.image_queue.push((image.clone(), o.clone()));
        }
        let layout = layouts.add(TextureAtlasLayout::from_grid(
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
}
