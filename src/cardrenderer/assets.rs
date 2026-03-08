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
	Texture(&'static str),
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
pub struct AssetCache {
	textures: HashMap<&'static str, Handle<Image>>
}

struct MaterialReference {
	material_id: u8
}

impl AssetCache {
	pub fn new<C: Assetable>(server: &AssetServer, set: &dyn AssetBand<C>) -> Self {
		let mut s = Self {
			textures: HashMap::new()
		};
		for img in set.predict_assets() {
			match img {
				AssetReference::Texture(name) => {HashMap::insert(&mut s.textures, name, server.load(name.to_owned() + ".png"));}
			}
		}
		return s;
	}

	pub fn get_assets(&self, ref_list: Vec<AssetReference>) -> Vec<Asset> {
		return ref_list.into_iter().map(|aref| {
			match aref {
				AssetReference::Texture(name) => Asset::Texture(self.textures[name].clone())
			}
		}).collect();
	}
}