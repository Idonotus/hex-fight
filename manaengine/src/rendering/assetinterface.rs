use std::any::TypeId;

use bevy::{
    app::{App, Plugin},
    asset::Handle,
    ecs::world::World,
    image::{Image, TextureAtlasLayout},
    math::UVec2,
    sprite_render::{Material2d, Material2dPlugin},
};

#[derive(PartialEq, Eq)]
pub enum AssetExpectations {
    ExistsAsImage,
    Texture,
    AtlasChild,
    Atlas { size: [u32; 2] },
}

pub struct ExpectedAssetRef {
    pub name: String,
    pub expectations: AssetExpectations,
}

impl ExpectedAssetRef {
    pub fn new<T: Into<String>>(name: T, expectations: AssetExpectations) -> Self {
        Self {
            name: name.into(),
            expectations,
        }
    }
}

#[derive(Debug)]
pub enum Asset {
    Texture(Handle<Image>),
    Palette(Handle<Image>, PaletteReference, PaletteReference),
    AtlasTexture(Handle<Image>, Handle<TextureAtlasLayout>, usize),
}

pub enum AssetReference {
    Texture(String),
    Palette(PaletteReference),
    AtlasIndex(String, usize),
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

pub trait AssetRequest {
    fn request_texture(&mut self, texture: String, address: usize) -> ();
    fn request_palette(&mut self, palette: &[RGBA], address: usize) -> ();
    fn pop(&mut self) -> Vec<Option<AssetReference>>;
    fn fill(&mut self, size: usize) -> Result<(), &'static str>;
    fn flush(self: Box<Self>, world: &mut World);
}

pub type PaletteReference = UVec2;
pub type RGBA = [u8; 4];

pub trait PaletteData: PaletteAllocator {
    fn add_palette(&mut self, palette: &[RGBA]) -> PaletteReference {
        let start = self.allocate(palette.len());
        let data: Vec<u8> = palette.iter().flatten().copied().collect();
        self.push_data(data, start);
        return self.get_ref_from_idx(start);
    }
    fn push_data(&mut self, data: Vec<u8>, start: usize);
}

pub trait PaletteAtlas: PaletteData {
    fn flush(&mut self, world: World) -> ();
}

pub trait PaletteAllocator {
    fn allocate(&mut self, size: usize) -> usize;
    fn get_size(&self) -> PaletteReference;
    fn get_ref_from_idx(&self, idx: usize) -> PaletteReference;
}
