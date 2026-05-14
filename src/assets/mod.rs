use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        entity::Entity,
        system::{ResMut, SystemState},
        world::World,
    },
    image::Image,
    math::{Vec3, primitives::Rectangle},
    transform::components::Transform,
};

use crate::assets::{
    cache::AssetCache,
    cardrender::{Assetable, RequestContext},
};

pub mod cache;
pub mod cardrender;
pub mod loader;
pub mod palette;

#[derive(Component)]
pub struct UICard;

pub mod batch {
    use bevy::{
        asset::Assets,
        ecs::{
            entity::Entity,
            system::{Commands, ResMut, SystemState},
            world::World,
        },
        image::Image,
        math::primitives::Rectangle,
    };

    use super::{
        UICard,
        cache::{Asset, AssetCache, AssetReference},
        cardrender::{Assetable, RequestContext},
    };

    pub fn render_cards<C: Assetable>(
        world: &mut World,
        cards: &Vec<C>,
        card_size: Rectangle,
    ) -> Vec<Entity> {
        let commands = world.commands();
        let entities = generate_bases(commands, cards.len());

        let (mut assets, images) =
            SystemState::<(ResMut<AssetCache>, ResMut<Assets<Image>>)>::new(world).get_mut(world);

        let context = RequestContext::new(&mut assets.palette, images);
        let card_requests = request_assets(cards, context);
        let card_info = obtain_assets(assets.as_ref(), card_requests);

        generate_cards(world, card_info, &entities, card_size);

        return entities;
    }

    fn generate_bases(mut commands: Commands, amount: usize) -> Vec<Entity> {
        return (0..amount).map(|_| commands.spawn(UICard).id()).collect();
    }

    fn request_assets<'a, C: Assetable>(
        cards: &'a Vec<C>,
        mut context: RequestContext,
    ) -> Vec<(&'a C, Vec<AssetReference>)> {
        let mut references = Vec::new();
        for c in cards {
            context.fill(c.get_asset_count()).unwrap();
            context = c.request_assets(context);
            references.push((c, context.pop().into_iter().map(|r| r.unwrap()).collect()));
        }
        return references;
    }

    fn obtain_assets<T>(
        cache: &AssetCache,
        mut requests: Vec<(T, Vec<AssetReference>)>,
    ) -> Vec<(T, Vec<Asset>)> {
        let mut assets = Vec::new();
        for (c, ref_list) in requests.drain(..) {
            assets.push((c, cache.get_assets(ref_list)));
        }
        return assets;
    }

    fn generate_cards<C: Assetable>(
        world: &mut World,
        card_info: Vec<(&C, Vec<Asset>)>,
        entities: &Vec<Entity>,
        card_size: Rectangle,
    ) {
        for (i, (c, assets)) in card_info.into_iter().enumerate() {
            let base_entity = entities[i].clone();
            c.generate_layers(world, card_size, base_entity, assets);
        }
    }
}

pub fn render_card<C: Assetable>(world: &mut World, card: &C, card_size: Rectangle) -> Entity {
    let mut commands = world.commands();
    let entity = commands.spawn(UICard).id();

    let (mut assets, images) =
        SystemState::<(ResMut<AssetCache>, ResMut<Assets<Image>>)>::new(world).get_mut(world);
    let mut context = RequestContext::new(&mut assets.palette, images);
    context.fill(card.get_asset_count()).unwrap();
    context = card.request_assets(context);
    let ref_list = context.pop().into_iter().map(|a| a.unwrap()).collect();
    let card_assets = assets.get_assets(ref_list);

    card.generate_layers(world, card_size, entity, card_assets);

    return entity;
}
