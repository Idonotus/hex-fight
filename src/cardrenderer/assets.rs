use bevy::{
    asset::RenderAssetUsages,
    platform::collections::{HashMap, HashSet},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension},
    sprite_render::{Material2d, Material2dPlugin},
};
use std::{any::TypeId, mem};

use crate::engine::cards::{AssignedBand, BandSet};

pub struct Details {
    pub name: String,
    pub description: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AssetReference {
    Texture(&'static str),
    Palette(PaletteReference),
}

pub enum Asset {
    Texture(Handle<Image>),
    Palette(Handle<Image>, PaletteReference, PaletteReference),
}

type PaletteReference = usize;
pub type RGBA = [u8; 4];

pub trait PaletteAtlas {
    fn add_palette(&mut self, image: &mut Image, palette: Vec<RGBA>) -> PaletteReference {
        let data: Vec<u8> = palette.into_iter().flatten().collect();
        return self.push_data(image, data);
    }
    fn push_data(&mut self, image: &mut Image, data: Vec<u8>) -> PaletteReference;
    fn new_reference(&mut self) -> PaletteReference;
    fn get_image(&self) -> Handle<Image>;
    fn get_size(&self) -> PaletteReference;
}

pub trait Assetable {
    fn get_details(&self) -> Details;
    fn generate_layers(
        &self,
        world: &mut World,
        commands: &mut Commands,
        card_size: Rectangle,
        base_entity: Entity,
        assets: Vec<Asset>,
    ) -> ();
    fn request_assets<'a>(&self, context: RequestContext<'a>) -> RequestContext<'a>;
}

pub trait AssetBand<'a, C>: AssignedBand<'a, C>
where
    C: Assetable,
{
    fn predict_assets(&self) -> HashSet<AssetReference>;
    fn predict_material(&self, cache: &mut MaterialCache) -> ();
}

impl<'a, C> AssetBand<'a, C> for Box<dyn AssetBand<'a, C> + 'a>
where
    C: Assetable,
    Box<dyn AssetBand<'a, C> + 'a>: AssetBand<'a, C>,
{
    fn predict_assets(&self) -> HashSet<AssetReference> {
        self.as_ref().predict_assets()
    }

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
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

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
        for band in self.iter() {
            band.predict_material(cache)
        }
    }
}

pub struct BasePalette {
    img: Handle<Image>,
    allocated: usize,
    size: usize,
}

impl BasePalette {
    fn new(img: Handle<Image>, size: usize) -> Self {
        Self {
            allocated: 0usize,
            img,
            size,
        }
    }
    fn gen_image(size: usize) -> Image {
        Image::new(
            Extent3d {
                width: size as u32,
                height: 1,
                ..Default::default()
            },
            TextureDimension::D2,
            vec![0; size * 4],
            bevy::render::render_resource::TextureFormat::Rgba8Uint,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        )
    }
}

impl PaletteAtlas for BasePalette {
    fn push_data(&mut self, image: &mut Image, data: Vec<u8>) -> PaletteReference {
        let image_data: &mut Vec<u8> = image.data.as_mut().unwrap();
        let initial_ref = self.allocated;
        let size: usize = data.len();

        self.allocated += size;

        image_data.splice(initial_ref..self.allocated, data);
        return self.allocated as PaletteReference;
    }

    fn get_image(&self) -> Handle<Image> {
        self.img.clone()
    }

    fn new_reference(&mut self) -> PaletteReference {
        return self.allocated as PaletteReference;
    }

    fn get_size(&self) -> PaletteReference {
        return self.size;
    }
}

#[derive(Resource)]
pub struct AssetCache {
    textures: HashMap<&'static str, Handle<Image>>,
    palette: BasePalette,
}

impl AssetCache {
    pub fn new<C: Assetable>(
        server: &AssetServer,
        set: &dyn AssetBand<C>,
        palette: BasePalette,
    ) -> Self {
        let mut s: AssetCache = Self {
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
                AssetReference::Palette(r) => {
                    Asset::Palette(self.palette.get_image(), r, self.palette.get_size())
                }
            })
            .collect();
    }
}

pub struct MaterialCache<'a> {
    app: &'a mut App,
    pub materials: Vec<TypeId>,
}

impl<'a> MaterialCache<'a> {
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

pub struct RequestContext<'a> {
    palettes: &'a mut dyn PaletteAtlas,
    references: Vec<Option<AssetReference>>,
    images: ResMut<'a, Assets<Image>>,
}

impl<'a> RequestContext<'a> {
    pub fn new(
        palettes: &'a mut dyn PaletteAtlas,
        asset_count: usize,
        images: ResMut<'a, Assets<Image>>,
    ) -> Self {
        let references: Vec<Option<AssetReference>> = (0..asset_count).map(|_| None).collect();
        Self {
            palettes,
            references,
            images,
        }
    }

    pub fn request_texture(&mut self, texture: &'static str, address: usize) -> () {
        self.references[address] = Some(AssetReference::Texture(texture));
    }

    pub fn request_palette(&mut self, palette: Vec<RGBA>, address: usize) -> () {
        let image = self.images.get_mut(&self.palettes.get_image()).unwrap();
        self.references[address] = Some(AssetReference::Palette(
            self.palettes.add_palette(image, palette),
        ));
    }

    pub fn pop(&mut self) -> Vec<Option<AssetReference>> {
        let mut r = Vec::new();
        mem::swap(&mut r, &mut self.references);
        return r;
    }
}
