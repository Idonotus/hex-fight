use std::mem::swap;

use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    assets::{
        cache::{Asset, AssetReference, MaterialCache},
        palette::{PaletteAtlas, RGBA},
    },
    engine::cards::{AssignedBand, BandSet},
};

pub struct Details {
    pub name: String,
    pub description: String,
}

pub trait Assetable {
    fn get_details(&self) -> Details;
    fn generate_layers(
        &self,
        world: &mut World,
        card_size: Rectangle,
        base_entity: Entity,
        assets: Vec<Asset>,
    ) -> ();
    fn request_assets<'a>(&self, context: RequestContext<'a>) -> RequestContext<'a>;
    fn get_asset_count(&self) -> usize;
}

pub trait AssetableGroup {
    fn predict_assets(&self) -> Vec<String>;
    fn predict_material(&self, cache: &mut MaterialCache) -> ();
}

impl<'a, Band, C> AssetableGroup for BandSet<'a, Band, C>
where
    Band: AssetableGroup + AssignedBand<'a, C>,
{
    fn predict_assets(&self) -> Vec<String> {
        self.iter().flat_map(|a| a.predict_assets()).collect()
    }

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
        for band in self.iter() {
            band.predict_material(cache)
        }
    }
}

pub struct RequestContext<'a> {
    palettes: &'a mut dyn PaletteAtlas,
    references: Vec<Option<AssetReference>>,
    images: ResMut<'a, Assets<Image>>,
}

impl<'a> RequestContext<'a> {
    pub fn new(palettes: &'a mut dyn PaletteAtlas, images: ResMut<'a, Assets<Image>>) -> Self {
        Self {
            palettes,
            references: Vec::new(),
            images,
        }
    }

    pub fn request_texture(&mut self, texture: String, address: usize) -> () {
        self.references[address] = Some(AssetReference::Texture(texture));
    }

    pub fn request_palette(&mut self, palette: Vec<RGBA>, address: usize) -> () {
        let image = self.images.get_mut(&self.palettes.get_image()).unwrap();
        self.references[address] = Some(AssetReference::Palette(
            self.palettes.add_palette(image, palette),
        ));
    }

    pub fn request_atlastexture(&mut self, texture: String, address: usize) -> () {
        self.references[address] = Some(AssetReference::AtlasTexture(texture));
    }

    pub fn pop(&mut self) -> Vec<Option<AssetReference>> {
        let mut r = Vec::new();
        swap(&mut r, &mut self.references);
        return r;
    }

    pub fn fill(&mut self, size: usize) -> Result<(), &'static str> {
        if self.references.len() > 0 {
            return Err("Context is not empty");
        }
        for _ in 0..size {
            self.references.push(None)
        }
        return Ok(());
    }
}
