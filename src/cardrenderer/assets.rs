use bevy::{
    asset::RenderAssetUsages,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension},
    sprite_render::{Material2d, Material2dPlugin},
};
use std::any::TypeId;

use crate::engine::cards::{AssignedBand, BandSet};

pub struct Details {
    pub name: String,
    pub description: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AssetReference {
    Texture(&'static str),
    Palette(u16),
}

pub enum Asset {
    Texture(Handle<Image>),
    Palette(Handle<Image>, u16),
}

type RGBA = [u8; 4];

trait PaletteAtlas {
    fn add_pallete(&mut self, images: Assets<Image>, palette: Vec<RGBA>) -> () {
        let data: Vec<u8> = palette.into_iter().flatten().collect();
        self.push_data(images, data);
    }
    fn push_data(&mut self, images: Assets<Image>, data: Vec<u8>) -> ();
    fn new_reference(&mut self) -> u32;
    fn get_image(&self) -> Handle<Image>;
}

pub trait Assetable {
    fn get_details(&self) -> Details;
    fn generate_layers(
        &self,
        world: &mut World,
        card_size: (),
        base_entity: Entity,
        assets: Vec<Asset>,
    ) -> ();
    fn request_assets(&self, palette: u16) -> Vec<AssetReference>;
    fn request_palette(&self) -> Vec<Color>;
}

pub trait AssetBand<'a, C>: AssignedBand<'a, C>
where
    C: Assetable,
{
    fn predict_assets(&self) -> HashSet<AssetReference>;
    fn predict_material(&self, cache: &MaterialCache) -> ();
}

impl<'a, C> AssetBand<'a, C> for Box<dyn AssetBand<'a, C> + 'a>
where
    C: Assetable,
    Box<dyn AssetBand<'a, C> + 'a>: AssetBand<'a, C>,
{
    fn predict_assets(&self) -> HashSet<AssetReference> {
        self.as_ref().predict_assets()
    }

    fn predict_material(&self, cache: &MaterialCache) -> () {
        self.as_ref().predict_material(cache)
    }
}

impl<'a, Band, C> AssetBand<'a, C> for BandSet<'a, Band, C>
where
    C: Assetable,
    Band: AssetBand<'a, C>,
{
    fn predict_assets(&self) -> HashSet<AssetReference> {
        HashSet::from_iter(self.iter().flat_map(|a| a.predict_assets()))
    }

    fn predict_material(&self, cache: &MaterialCache) -> () {
        for band in self.iter() {
            band.predict_material(cache)
        }
    }
}

struct BasePalette {
    img: Handle<Image>,
    allocated: usize,
}

impl BasePalette {
    fn new(img: Handle<Image>) -> Self {
        Self {
            allocated: 0usize,
            img,
        }
    }
    fn gen_image(size: u32) -> Image {
        Image::new(
            Extent3d {
                width: size,
                height: 1,
                ..Default::default()
            },
            TextureDimension::D2,
            vec![0; (size * 4).try_into().unwrap()],
            bevy::render::render_resource::TextureFormat::Rgba8Uint,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
    }
}

impl PaletteAtlas for BasePalette {
    fn push_data(&mut self, mut images: Assets<Image>, data: Vec<u8>) -> () {
        let image_data: &mut Vec<u8> = images.get_mut(&self.img).unwrap().data.as_mut().unwrap();
        let size: usize = data.len();

        image_data.splice(self.allocated..(self.allocated + size), data);
        self.allocated += size;
    }

    fn get_image(&self) -> Handle<Image> {
        self.img.clone()
    }

    fn new_reference(&mut self) -> u32 {
        let a: u32 = self.allocated.try_into().unwrap();
        return a / 4;
    }
}

#[derive(Resource)]
pub struct AssetCache<A: PaletteAtlas> {
    textures: HashMap<&'static str, Handle<Image>>,
    palette: A,
}

impl<A: PaletteAtlas> AssetCache<A> {
    pub fn new<C: Assetable>(server: &AssetServer, set: &dyn AssetBand<C>, palette: A) -> Self {
        let mut s: AssetCache<A> = Self {
            textures: HashMap::new(),
            palette,
        };
        for img in set.predict_assets() {
            match img {
                AssetReference::Texture(name) => {
                    HashMap::insert(&mut s.textures, name, server.load(name.to_owned() + ".png"));
                }
                AssetReference::Palette(_) => panic!("Cannot predict palette from band."),
            }
        }
        return s;
    }

    pub fn get_assets(&self, ref_list: Vec<AssetReference>) -> Vec<Asset> {
        return ref_list
            .into_iter()
            .map(|aref| match aref {
                AssetReference::Texture(name) => Asset::Texture(self.textures[name].clone()),
                AssetReference::Palette(r) => Asset::Palette(self.palette.get_image(), r),
            })
            .collect();
    }
}

struct MaterialCache<'a> {
    pub app: &'a mut App,
    pub materials: Vec<TypeId>,
}

impl<'a> MaterialCache<'a> {
    fn add_mat<T>(&mut self) -> ()
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
