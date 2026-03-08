use bevy::color::LinearRgba;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;

use crate::content::*;
use super::{
	assets::{
		AssetBand, AssetReference, Assetable, Details, Asset
	},
	materials::RecolourMaterial
};
use crate::engine::{
	cards::{
		AssignedBand,
		Stacks
	},
	colors::Color as CardColor
};

const CARD_BASE: &'static str = "card-base";
const WHITE: LinearRgba = LinearRgba::new(1.0, 1.0, 1.0, 1.0);
const BLACK: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 1.0);

impl Into<LinearRgba> for CardColor {
	fn into(self) -> LinearRgba {
		LinearRgba { red: <u8 as Into<f32>>::into(self.r) / 256.0, green: <u8 as Into<f32>>::into(self.g) / 256.0, blue: <u8 as Into<f32>>::into(self.b) / 256.0, alpha: 1.0 }
	}
}

impl<'a, C: Assetable> AssetBand<'a, C> for AllColorBand where AllColorBand: AssignedBand<'a, C> {
	fn predict_assets(&self) -> HashSet<AssetReference> {
		HashSet::from_iter(
			vec![
				AssetReference::Texture(CARD_BASE)
			]
		)
	}
}

impl<'a> Assetable for SimpleCard {
	fn get_details(&self) -> super::assets::Details {
		Details {
			name: format!("{:?} of {}", self.get_value(), self.get_color().unwrap()),
			description: "A normal card".to_owned()
		}
	}
	fn generate_layers(&self, mut assets: Vec<Asset>) -> Vec<Box<dyn std::any::Any>> {
		let Asset::Texture(img) = assets.remove(0) else {
			return Vec::new();
		};
		let Some(c) = self.get_color() else {
			return Vec::new();
		};
		let borders = if c.get_value() > 0.5 {BLACK} else {WHITE};

		vec![
			Box::new(
				RecolourMaterial {
					pallete: vec![
						borders,
						c.into()
					],
					color_texture: Some(img)
				}
			)
		]
	}

	fn request_assets(&self) -> Vec<AssetReference> {
		vec![AssetReference::Texture(CARD_BASE)]
	}
}

impl<'a, Band: AssetBand<'a, C>, C: Assetable> AssetBand<'a, C> for PluralBand<'a, Band, C>{
	fn predict_assets(&self) -> bevy::platform::collections::HashSet<AssetReference> {
		self.0.predict_assets()
	}
}