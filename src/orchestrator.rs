use std::sync::Arc;

use crate::{
    assets::{
        batch::render_cards, cache::AssetInterface, cardrender::AssetableGroup, loader,
        palette::PaletteAtlas,
    },
    cardmenu::{CardGroup, WidthLayout, display_groups, origins::CardUIOrigins},
    engine::cards::{DeckId, does_stack},
};

use super::{
    assets::{
        cache::{Asset as GameAsset, AssetCache, MaterialCache},
        cardrender::{Assetable, Details, RequestContext},
        palette::BasePalette,
    },
    content::basic_cards::{AllColorBand, AllColorPlugin},
    engine::{
        Game,
        cards::{AssignedBand, BandSet, CardValue, Stacks},
        colors::{Color, ColorComparison},
    },
};
use bevy::{ecs::system::SystemState, input::mouse::MouseWheel, prelude::*};
use dyn_clone::DynClone;

trait Card: Stacks + Assetable + DynClone + Send + Sync {}

dyn_clone::clone_trait_object!(Card);

type CardBox<'a> = Box<dyn Card + 'a>;

impl<'a> Assetable for CardBox<'a> {
    fn get_details(&self) -> Details {
        return self.as_ref().get_details();
    }

    fn generate_layers(
        &self,
        world: &mut World,
        card_size: Rectangle,
        base_entity: Entity,
        assets: Vec<GameAsset>,
    ) -> () {
        return self
            .as_ref()
            .generate_layers(world, card_size, base_entity, assets);
    }

    fn request_assets<'b>(&self, context: RequestContext<'b>) -> RequestContext<'b> {
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
impl<T: Stacks + Assetable + Clone + Sync + Send> Card for T {}

impl<'a> AssignedBand<'a, CardBox<'a>> for AllColorBand {
    fn generate_card(&mut self, c_id: DeckId) -> CardBox<'a> {
        Box::new(self.generate_card(c_id))
    }
}

#[derive(Component)]
struct ScrollMenu {}

#[derive(Component)]
struct Selected {}

const PALETTE_SIZE: UVec2 = UVec2 { x: 800, y: 400 };
const CARD_SIZE: Vec3 = Vec3 {
    x: 90.0,
    y: 140.0,
    z: 0.0,
};

fn setup_game(world: &mut World) {
    let b = BandSet::new(vec![AllColorBand::new(10)]);
    let mut images = world.resource_mut::<Assets<Image>>();
    let img = images.reserve_handle();
    images.insert(&img, BasePalette::gen_image(PALETTE_SIZE));
    let p = BasePalette::new(img, PALETTE_SIZE);

    let mut cache = AssetCache::new(p);
    loader::load_pack_index(
        loader::path_to_abs("base_pack/record.json", None),
        &mut cache,
    )
    .unwrap();
    let (server, layouts) =
        SystemState::<(ResMut<AssetServer>, ResMut<Assets<TextureAtlasLayout>>)>::new(world)
            .get_mut(world);

    cache.load(AssetInterface { server, layouts }, b.predict_assets());

    world.insert_resource(cache);
    let mut game = Game::new(2, Box::new(rand::rng()), b);
    game.deal(2000);
    game.top_card = game.draw_card();
    world.insert_non_send_resource(game);
}

fn test_system(world: &mut World) {
    // Ge t the card
    let (game,) = SystemState::<(NonSend<Game<AllColorBand, CardBox>>,)>::new(world).get_mut(world);
    let cardinfo: Vec<CardBox> = game
        .get_current_player()
        .hand
        .iter()
        .map(|c| c.clone())
        .collect();
    let cur_turn: usize = game.order.get_turn();

    let vstacks = game.get_valid_stacks_for_player(cur_turn, &|base, head| {
        does_stack(base, head, &game.comparison)
    });

    let mut cards = render_cards(
        world,
        &cardinfo,
        Rectangle::from_size(Vec2 { x: 90.0, y: 140.0 }),
    );

    let (mut origins,) = SystemState::<(ResMut<CardUIOrigins>,)>::new(world).get_mut(world);

    for idx in 0..cardinfo.len() {
        origins.register_card(
            cards[idx],
            crate::cardmenu::origins::CardOrigin::Hand(cur_turn, idx),
        );
    }

    let mut commands = world.commands();

    let playable_group = commands.spawn(()).id();
    let mut playable_cards = Vec::new();
    let unplayable_group = commands.spawn(()).id();

    commands
        .spawn((
            ScrollMenu {},
            Selected {},
            Transform::from_scale(Vec3::splat(1.0)),
        ))
        .add_children(&[playable_group, unplayable_group]);

    for index in vstacks.into_iter().rev() {
        playable_cards.push(cards.remove(index));
    }

    let positionerbase = WidthLayout {
        width: 14,
        card_size: CARD_SIZE,
    };

    let groups = vec![
        (
            playable_group,
            CardGroup {
                name: "Playable cards".to_owned(),
                layout: Box::new(positionerbase),
                cards: playable_cards,
            },
        ),
        (
            unplayable_group,
            CardGroup {
                name: "Other cards".to_owned(),
                layout: Box::new(positionerbase),
                cards,
            },
        ),
    ];

    display_groups(commands, groups, 140.0);

    world.flush();
}

const SCROLL_SCALE: f32 = 10.0;
const EXTRA_MODIFIER: f32 = 10.0;

fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    move_map: Query<(&mut Transform,), (With<ScrollMenu>, With<Selected>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut delta = Vec2::splat(0.0);
    for mouse_wheel in mouse_wheel_reader.read() {
        delta += Vec2::new(mouse_wheel.x, mouse_wheel.y);
    }

    delta *= -SCROLL_SCALE;

    if keyboard_input.pressed(KeyCode::ShiftLeft) {
        delta *= EXTRA_MODIFIER;
    }

    if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        std::mem::swap(&mut delta.x, &mut delta.y);
    }

    for (mut t,) in move_map.into_iter() {
        t.translation += vec3(delta.x, delta.y, 0.0);
    }
}

fn loading(
    mut commands: Commands,
    server: Res<AssetServer>,
    images: ResMut<Assets<Image>>,
    mut cache: ResMut<AssetCache>,
) {
    let done = cache.loading(server, images);
    if done {
        println!("Done loading!");
        commands.set_state(GameState::InGame);
    }
}

pub struct GamePlugin;

#[derive(Component)]
struct Hand {
    data: Vec<Arc<dyn Card>>,
}

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
        app.init_state::<GameState>()
            .add_systems(
                Startup,
                (setup_game, crate::cardmenu::create_menu_resources),
            )
            .add_systems(OnEnter(GameState::InGame), test_system)
            .add_systems(
                Update,
                (
                    loading.run_if(in_state(GameState::Loading)),
                    send_scroll_events,
                ),
            );
    }
}

#[derive(Debug, States, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    NotInGame,
    #[default]
    Loading,
    InGame,
}
