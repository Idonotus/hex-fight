#[rustfmt::skip()]
mod defcolors {
    use super::super::assets::RGBA;
    pub const WHITE: RGBA = [255, 255, 255, 255];
    pub const BLACK: RGBA = [0, 0, 0, 255];
}

use defcolors::*;

#[rustfmt::skip()]
use super::assets::{Asset, AssetBand, AssetReference, Assetable, Details, MaterialCache, RGBA};

use crate::cardrenderer::assets::RequestContext;
use crate::content::*;
use crate::engine::{
    cards::{AssignedBand, Stacks},
    colors::Color as CardColor,
};

use bevy::{
    platform::collections::HashSet,
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct RecolourMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Option<Handle<Image>>,
    #[texture(2)]
    #[sampler(3)]
    palette: Option<Handle<Image>>,
    #[uniform(5)]
    offset: f32,
}

impl Material2d for RecolourMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/color_map.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Mask(0.5)
    }
}

const CARD_BASE: &'static str = "card-base";

impl Into<RGBA> for CardColor {
    fn into(self) -> RGBA {
        return [self.r, self.g, self.b, 255];
    }
}

impl<'a, C: Assetable> AssetBand<'a, C> for AllColorBand
where
    AllColorBand: AssignedBand<'a, C>,
{
    fn predict_assets(&self) -> HashSet<AssetReference> {
        HashSet::from_iter(vec![AssetReference::Texture(CARD_BASE)])
    }

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
        cache.add_mat::<RecolourMaterial>();
    }
}

impl Assetable for SimpleCard {
    fn get_details(&self) -> super::assets::Details {
        Details {
            name: format!("{:?} of {}", self.get_value(), self.get_color().unwrap()),
            description: "A normal card".to_owned(),
        }
    }

    fn generate_layers(
        &self,
        world: &mut World,
        commands: &mut Commands,
        card_size: Rectangle,
        base_entity: Entity,
        mut assets: Vec<Asset>,
    ) -> () {
        let Asset::Texture(img) = assets.remove(0) else {
            return;
        };
        let Asset::Palette(palette, offset) = assets.remove(0) else {
            return;
        };

        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = Mesh2d(meshes.add(card_size));

        let mut materials = world.resource_mut::<Assets<RecolourMaterial>>();
        let mat = MeshMaterial2d(materials.add(RecolourMaterial {
            palette: Some(palette),
            texture: Some(img),
            offset: offset as f32,
        }));
        let e = commands.spawn((mat, mesh)).id();
        commands.entity(base_entity).add_child(e);
    }

    fn request_assets<'a>(&self, mut context: RequestContext<'a>) -> RequestContext<'a> {
        context.request_texture(CARD_BASE, 0);

        let Some(c) = self.get_color() else {
            return context;
        };

        let borders = if c.get_value() > 0.5 { BLACK } else { WHITE };

        context.request_palette(vec![borders, c.into()], 1);

        return context;
    }
}

impl<'a, Band: AssetBand<'a, C>, C: Assetable> AssetBand<'a, C> for PluralBand<'a, Band, C> {
    fn predict_assets(&self) -> bevy::platform::collections::HashSet<AssetReference> {
        self.0.predict_assets()
    }

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
        self.0.predict_material(cache);
    }
}
