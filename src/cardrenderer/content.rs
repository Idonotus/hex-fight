use bevy::color::LinearRgba;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d};

use crate::content::*;
use crate::cardrenderer::assets::{
	AssetBand, AssetReference, Assetable, Details, Asset
};
use crate::engine::{
	cards::{
		AssignedBand,
		Stacks
	},
	colors::Color as CardColor
};

const CARD_BASE: String = "card-base".to_owned();
const WHITE: LinearRgba = LinearRgba::new(1.0, 1.0, 1.0, 1.0);
const BLACK: LinearRgba = LinearRgba::new(0.0, 0.0, 0.0, 1.0);

impl Into<LinearRgba> for CardColor {
	fn into(self) -> LinearRgba {
		LinearRgba { red: self.r.into() / 256.0, green: self.r.into() / 256.0, blue: self.b.into() / 256.0, alpha: 1.0 }
	}
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct RecolourMaterial {
    #[uniform(0)]
    pallete: Vec<LinearRgba>,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
}

impl Material2d for RecolourMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/color_map.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Mask(0.5)
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
	fn generate_layers(&self, assets: Vec<super::assets::Asset>) -> Vec<Box<dyn std::any::Any>> {
		let Asset::Texture(img) = assets[0] else {
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