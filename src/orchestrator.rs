use crate::{
    cardrenderer::assets::{
        AssetCache, AssetReference, AssetableGroup, BasePalette, MaterialCache, RequestContext,
    },
    content::{AllColorBand, AllColorPlugin, SimpleCard},
    engine::{
        Game,
        cards::{AssignedBand, BandSet},
    },
};

use super::{
    cardrenderer::assets::{Asset as GameAsset, Assetable},
    engine::{
        cards::{CardValue, Stacks},
        colors::{Color, ColorComparison},
    },
};
use bevy::{ecs::system::SystemState, prelude::*};
use dyn_clone::DynClone;

#[derive(Component)]
struct UICard {}

trait Card: Stacks + Assetable + DynClone {}

dyn_clone::clone_trait_object!(Card);

type CardBox<'a> = Box<dyn Card + 'a>;

impl<'a> Assetable for CardBox<'a> {
    fn get_details(&self) -> crate::cardrenderer::assets::Details {
        return self.as_ref().get_details();
    }

    fn generate_layers(
        &self,
        world: &mut World,
        card_size: Rectangle,
        base_entity: Entity,
        assets: Vec<crate::cardrenderer::assets::Asset>,
    ) -> () {
        return self
            .as_ref()
            .generate_layers(world, card_size, base_entity, assets);
    }

    fn request_assets<'b>(
        &self,
        context: crate::cardrenderer::assets::RequestContext<'b>,
    ) -> crate::cardrenderer::assets::RequestContext<'b> {
        return self.as_ref().request_assets(context);
    }

    fn get_asset_count(&self) -> usize {
        return self.as_ref().get_asset_count();
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
impl<T: Stacks + Assetable + Clone> Card for T {}

impl<'a> AssignedBand<'a, CardBox<'a>> for AllColorBand {
    fn generate_card(&mut self, c_id: u64) -> CardBox<'a> {
        Box::new(self.generate_card(c_id))
    }
}

fn setup_game(world: &mut World) {
    let b = BandSet::new(vec![AllColorBand::new(10)]);

    let mut i = world.resource_mut::<Assets<Image>>();
    let img = i.reserve_handle();
    i.insert(&img, BasePalette::gen_image(900));
    let p = BasePalette::new(img, 900);

    let server = world.resource::<AssetServer>();
    world.insert_resource(AssetCache::new::<SimpleCard>(server, &b, p));
    let mut game = Game::new(2, Box::new(rand::rng()), b);
    game.deal(14);
    world.insert_non_send_resource(game);
}

fn test_system(world: &mut World) {
    // Ge t the card
    let (mut assets, images, game) = SystemState::<(
        ResMut<AssetCache>,
        ResMut<Assets<Image>>,
        NonSend<Game<AllColorBand, CardBox>>,
    )>::new(world)
    .get_mut(world);
    let mut context = RequestContext::new(&mut assets.palette, images);
    let mut cardinfo: Vec<CardBox> = game
        .get_current_player()
        .hand
        .iter()
        .map(|c| c.clone())
        .collect();
    let mut arcardinfo: Vec<Vec<AssetReference>> = Vec::new();

    for card in cardinfo.iter() {
        context.fill(card.get_asset_count()).unwrap();
        context = card.request_assets(context);
        let c = context.pop().into_iter().map(|a| a.unwrap()).collect();
        arcardinfo.push(c);
    }

    let mut acardinfo: Vec<Vec<GameAsset>> = Vec::new();
    for assetref in arcardinfo.drain(..) {
        acardinfo.push(assets.get_assets(assetref));
    }

    // MAKEIT NOW!!!!
    let mut commands = world.commands();
    let mut ecardinfo: Vec<Entity> = Vec::new();
    for i in 0..cardinfo.len() {
        ecardinfo.push(
            commands
                .spawn((
                    UICard {},
                    Transform::default()
                        .with_scale(Vec3::splat(2.0))
                        .with_translation(Vec3 {
                            x: (i as f32 - 6.0) * 180.0,
                            y: 0.0,
                            z: 0.0,
                        }),
                ))
                .id(),
        )
    }

    for _ in 0..cardinfo.len() {
        let base_entity = ecardinfo.pop().unwrap();
        let cardassets = acardinfo.pop().unwrap();
        let card = cardinfo.pop().unwrap();
        card.generate_layers(world, Rectangle::new(90.0, 140.0), base_entity, cardassets);
    }

    world.flush();
}

pub struct GamePlugin;

impl GamePlugin {
    fn insert_plugins(&self, app: &mut App) {
        let mut m = MaterialCache::new(app);
        let c = AllColorPlugin {};
        c.predict_material(&mut m);
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        self.insert_plugins(app);
        app.add_systems(Startup, (setup_game, test_system).chain());
    }
}
