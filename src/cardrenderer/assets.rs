use bevy::{
	asset::RenderAssetUsages, platform::collections::{HashMap, HashSet}, prelude::*, render::render_resource::{Extent3d, TextureDimension}, sprite_render::{Material2d, Material2dPlugin}
};
use std::any::{Any, TypeId};

use crate::engine::{
	cards::{
		AssignedBand,
		BandSet
	},
};

pub struct Details {
	pub name: String,
	pub description: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AssetReference {
	Texture(&'static str),
	Palette(Vec<crate::engine::colors::Color>, u16),
}

pub enum Asset {
	Texture(Handle<Image>),
	Palette(Handle<Image>, u16)
}

trait PaletteAtlas: Into<Image> {
	fn add_pallete(&mut self) -> ();
	fn new_reference(&mut self) -> ();
}

pub trait Assetable {
	fn get_details(&self) -> Details;
	fn generate_layers(&self, world: &mut World, card_size: (), base_entity: Entity, assets: Vec<Asset>) -> ();
	fn request_assets(&self, palette: u16) -> Vec<AssetReference>;
	fn request_palette(&self) -> Vec<Color>;
}

pub trait AssetBand<'a, C>: AssignedBand<'a, C>
where C: Assetable {
	fn predict_assets(&self) -> HashSet<AssetReference>;
	fn predict_material(&self, cache: MaterialCache) -> ();
}

impl<'a, C> AssetBand<'a, C> for Box<dyn AssetBand<'a, C> + 'a>
where
	C: Assetable,
	Box<dyn AssetBand<'a, C> + 'a>: AssetBand<'a, C> {
	fn predict_assets(&self) -> HashSet<AssetReference> {
		self.as_ref().predict_assets()
	}

	fn predict_material(&self, cache: MaterialCache) -> () {
		self.as_ref().predict_material(cache)
	}
}

impl<'a, Band, C> AssetBand<'a, C> for BandSet<'a, Band, C>
where 
	C: Assetable,
	Band: AssetBand<'a, C> {
	fn predict_assets(&self) -> HashSet<AssetReference> {
		HashSet::from_iter(self.iter().flat_map(|a| a.predict_assets()))
	}

	fn predict_material(&self, cache: MaterialCache) -> () {
		for band in self.iter() {
			band.predict_material(cache)
		}
	}
}

struct BasePalette {
	data: Vec<u8>
}

impl BasePalette {
	fn new() -> Self {
		Self {
			data: Vec::new()
		}
	}
}

impl Into<Image> for BasePalette {
	fn into(self) -> Image {
		let c: u32 = self.data.len().into() / 4;
		Image::new(Extent3d {width: 1, height: c, ..Default::default()}, TextureDimension::D2, self.data, bevy::render::render_resource::TextureFormat::Rgba8Uint, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD)
	}
}

#[derive(Resource)]
pub struct AssetCache {
	textures: HashMap<&'static str, Handle<Image>>,
	palette: BasePalette,
}

impl AssetCache {
	pub fn new<C: Assetable>(server: &AssetServer, set: &dyn AssetBand<C>) -> Self {
		let mut s: AssetCache = Self {
			textures: HashMap::new(),
			palette: todo!()
		};
		for img in set.predict_assets() {
			match img {
				AssetReference::Texture(name) => {HashMap::insert(&mut s.textures, name, server.load(name.to_owned() + ".png"));},
				AssetReference::Palette(_) => todo!(),
			}
		}
		return s;
	}

	pub fn get_assets(&self, ref_list: Vec<AssetReference>) -> Vec<Asset> {
		return ref_list.into_iter().map(|aref| {
			match aref {
				AssetReference::Texture(name) => Asset::Texture(self.textures[name].clone()),
				AssetReference::Palette(_) => todo!(),
			}
		}).collect();
	}
}

struct MaterialCache<'a> {
	pub app: &'a mut App,
	pub materials: Vec<TypeId>
}

impl<'a> MaterialCache<'a> {
	fn add_mat<T>(&mut self) -> ()
	where
		T: Material2d,
		Material2dPlugin<T>: Plugin
	{
		let t = TypeId::of::<T>();
		if self.materials.iter().any(|r| *r == t) {
			return;
		}
		self.app.add_plugins((Material2dPlugin::<T>::default(), ));
	}
}
