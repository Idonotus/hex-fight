use bevy::{
	platform::collections::{HashSet, HashMap},
	prelude::*
};
use std::any::Any;

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
	Texture(String),
}

pub enum Asset {
	Texture(Handle<Image>)
}

pub trait Assetable {
	fn get_details(&self) -> Details;
	fn generate_layers(&self, assets: Vec<Asset>) -> Vec<Box<dyn Any>>;
	fn request_assets(&self) -> Vec<AssetReference>;
}

pub trait AssetBand<'a, C>: AssignedBand<'a, C>
where C: Assetable {
	fn predict_assets(&self) -> HashSet<AssetReference>;
}

impl<'a, C> AssetBand<'a, C> for Box<dyn AssetBand<'a, C> + 'a>
where
	C: Assetable,
	Box<dyn AssetBand<'a, C> + 'a>: AssetBand<'a, C> {
	fn predict_assets(&self) -> HashSet<AssetReference> {
		self.as_ref().predict_assets()
	}
}

impl<'a, Band, C> AssetBand<'a, C> for BandSet<'a, Band, C>
where 
	C: Assetable,
	Band: AssetBand<'a, C> {
	fn predict_assets(&self) -> HashSet<AssetReference> {
		HashSet::from_iter(self.iter().flat_map(|a| a.predict_assets()))
	}
}

#[derive(Resource)]
struct AssetCache {
	textures: HashMap<String, Handle<Image>>
}

impl AssetCache {
	pub fn new(server: Res<AssetServer>, set: &dyn AssetBand) -> Self {
		let s = Self {
			textures: HashMap::new()
		};
		for img in set.predict_assets() {
			match img {
				AssetReference::Texture(name) => {s.textures[name] = server.load(name + ".png")}
			}
		}
		return s;
	}

	pub fn get_assets(&self, ref_list: Vec<AssetReference>) -> Vec<Asset> {
		return ref_list.into_iter().map(|aref| {
			match aref {
				AssetReference::Texture(name) => Asset::Texture(self.textures[name])
			}
		}).collect();
	}
}