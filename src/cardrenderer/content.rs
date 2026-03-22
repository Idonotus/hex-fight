#[rustfmt::skip()]
mod defcolors {
    use super::super::assets::RGBA;
    pub const WHITE: RGBA = [255, 255, 255, 255];
    pub const BLACK: RGBA = [0, 0, 0, 255];
}

use defcolors::*;

#[rustfmt::skip()]
use super::assets::{
    Asset, AssetPreload, Assetable, AssetableGroup, DescriptorOverride, Details, MaterialCache,
    RGBA, RequestContext,
};

use crate::content::*;
use crate::engine::{
    cards::{AssignedBand, Stacks},
    colors::Color as CardColor,
};

use bevy::{
    platform::collections::HashSet,
    prelude::*,
    render::render_resource::AsBindGroup,
    render::render_resource::TextureFormat,
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d},
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct RecolourMaterial {
    #[texture(0, dimension = "2d", sample_type = "u_int")]
    texture: Option<Handle<Image>>,
    #[uniform(1)]
    size: UVec2,
    #[texture(2, dimension = "1d")]
    palette: Option<Handle<Image>>,
    #[uniform(3)]
    offset: u32,
    #[uniform(4)]
    cap: u32,
}

impl Material2d for RecolourMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/color_map.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const CARD_BASE: &'static str = "card-base";
const APLUGIN: AllColorPlugin = AllColorPlugin {};

impl Into<RGBA> for CardColor {
    fn into(self) -> RGBA {
        return [self.r, self.g, self.b, 255];
    }
}
impl AssetableGroup for AllColorPlugin {
    fn predict_assets(&self) -> HashSet<AssetPreload> {
        HashSet::from_iter(vec![AssetPreload::Texture(
            CARD_BASE,
            DescriptorOverride {
                format: Some(TextureFormat::Rgba8Uint),
            },
        )])
    }

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
        cache.add_mat::<RecolourMaterial>();
    }
}

impl AssetableGroup for AllColorBand {
    fn predict_assets(&self) -> HashSet<AssetPreload> {
        APLUGIN.predict_assets()
    }

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
        APLUGIN.predict_material(cache);
    }
}

impl Assetable for SimpleCard {
    fn get_details(&self) -> super::assets::Details {
        Details {
            name: format!("{:?} of {}", self.get_value(), self.get_color().unwrap()),
            description: "A normal card".to_owned(),
        }
    }

    fn get_asset_count(&self) -> usize {
        2
    }

    fn generate_layers(
        &self,
        world: &mut World,
        card_size: Rectangle,
        base_entity: Entity,
        mut assets: Vec<Asset>,
    ) -> () {
        let Asset::Texture(img) = assets.remove(0) else {
            return;
        };
        let Asset::Palette(palette, offset, cap) = assets.remove(0) else {
            return;
        };

        let images = world.resource::<Assets<Image>>();
        let size = images.get(&img).unwrap().size();

        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = Mesh2d(meshes.add(card_size));

        let mut materials = world.resource_mut::<Assets<RecolourMaterial>>();
        let mat = MeshMaterial2d(materials.add(RecolourMaterial {
            palette: Some(palette),
            texture: Some(img),
            offset: offset as u32,
            cap: cap as u32,
            size,
        }));
        let mut commands = world.commands();
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

impl<'a, Band: AssignedBand<'a, C> + AssetableGroup, C: Assetable> AssetableGroup
    for PluralBand<'a, Band, C>
{
    fn predict_assets(&self) -> bevy::platform::collections::HashSet<AssetPreload> {
        self.0.predict_assets()
    }

    fn predict_material(&self, cache: &mut MaterialCache) -> () {
        self.0.predict_material(cache);
    }
}
