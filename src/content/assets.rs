use bevy::platform::collections::HashSet;

use crate::engine::{
	cards::{
		AssignedBand,
		BandSet
	},
};

pub struct CardAssets {
	pub name: &'static str,
	pub description: &'static str,

}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum AssetReference {
	Texture(String),
	Material(String)
}

pub trait Assetable {
	fn get_assets(&self) -> CardAssets;
}

pub trait AssetBand<'a, C>: AssignedBand<'a, C>
where C: Assetable {
	fn predict_assets(&self) -> HashSet<AssetReference> {
		panic!()
	}
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
		let set = HashSet::new();
		for i in self.iter() {
			set.union(&(i.predict_assets()));
		}
		return set;
	}
}