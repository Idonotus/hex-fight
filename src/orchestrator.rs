use std::any::{Any, TypeId};

use crate::{
    cardrenderer::{assets::AssetCache, materials::RecolourMaterial},
    content::{AllColorBand, SimpleCard},
    engine::{
        Game,
        cards::{AssignedBand, BandSet},
    },
};

use super::{
    cardrenderer::assets::Assetable,
    engine::{
        cards::{CardValue, Stacks},
        colors::{Color, ColorComparison},
    },
};
use bevy::prelude::*;

trait Card: Stacks + Assetable {}
type CardBox<'a> = Box<dyn Card + 'a>;

impl<'a> Assetable for CardBox<'a> {
    fn generate_layers(
        &self,
        assets: Vec<crate::cardrenderer::assets::Asset>,
    ) -> Vec<Box<dyn std::any::Any>> {
        self.as_ref().generate_layers(assets)
    }
    fn get_details(&self) -> crate::cardrenderer::assets::Details {
        self.as_ref().get_details()
    }
    fn request_assets(&self) -> Vec<crate::cardrenderer::assets::AssetReference> {
        self.as_ref().request_assets()
    }
}
impl<'a> Stacks for CardBox<'a> {
    fn can_get_stacked(&self, head: &dyn Stacks, color_comparason: &ColorComparison) -> bool {
        self.as_ref().can_get_stacked(head, color_comparason)
    }
    fn can_stack_onto(&self, base: &dyn Stacks, color_comparason: &ColorComparison) -> bool {
        self.as_ref().can_stack_onto(base, color_comparason)
    }

    fn get_color(&self) -> Option<Color> {
        self.as_ref().get_color()
    }
    fn get_value(&self) -> CardValue {
        self.as_ref().get_value()
    }
    fn get_stacking_priority(&self) -> i16 {
        self.as_ref().get_stacking_priority()
    }
}
impl<T: Stacks + Assetable> Card for T {}

impl<'a> AssignedBand<'a, CardBox<'a>> for AllColorBand {
    fn generate_card(&mut self, c_id: u64) -> CardBox<'a> {
        Box::new(self.generate_card(c_id))
    }
}

fn setup_game(world: &mut World) {
    let b = BandSet::new(vec![AllColorBand::new(10)]);
    let server = world.resource::<AssetServer>();
    world.insert_resource(AssetCache::new(server, &b));
    let game = Game::new(2, Box::new(rand::rng()), b);
    world.insert_non_send_resource(game);
}

fn test_system(
    mut commands: Commands,
    mut game: NonSendMut<Game<AllColorBand, CardBox>>,
    assets: ResMut<AssetCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<RecolourMaterial>>,
) {
    let card = game.draw_card().unwrap();
    let a = assets.get_assets(card.request_assets());
    let layers = card.generate_layers(a);
    let mut previous_child: Option<Entity> = None;
    for l in layers {
        let mat = l.downcast::<RecolourMaterial>();
        let omat = match mat {
            Ok(m) => m,
            Err(a) => {
                println!("{:#?} {:#?}", a.type_id(), TypeId::of::<RecolourMaterial>());
                continue;
            }
        };
        let e = commands
            .spawn((
                Mesh2d(meshes.add(Rectangle::default())),
                MeshMaterial2d(materials.add(*omat)),
                Transform::default().with_scale(Vec3::splat(128.)),
            ))
            .id();
        match previous_child {
            None => {}
            Some(p) => {
                commands.entity(p).add_child(e);
            }
        }
        previous_child = Some(e);
    }
}

pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_game, test_system).chain());
    }
}
