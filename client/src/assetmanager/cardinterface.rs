use std::mem::swap;

use bevy::{
    asset::Assets,
    ecs::{
        entity::Entity,
        resource::Resource,
        system::{Res, ResMut, SystemState},
        world::World,
    },
    image::Image,
    math::primitives::Rectangle,
};

use manaengine::rendering::{
    assetinterface::{
        AssetReference, AssetRequest, PaletteAllocator, PaletteData, PaletteReference, RGBA,
    },
    cardrender::Assetable,
};

use crate::assetmanager::{
    cache::{AssetCache, AssetContainer},
    palettes::PaletteReservations,
};

pub struct RequestContext {
    palette: PaletteReservations,
    buffer: Vec<u8>,
    offset: usize,
    references: Vec<Option<AssetReference>>,
}

impl RequestContext {
    pub fn new(cache: Res<AssetCache>) -> Self {
        let palette = cache.palette.allocator.clone();
        return Self {
            buffer: Vec::new(),
            references: Vec::new(),
            palette,
            offset: (palette.allocated.x + palette.allocated.y * palette.size.x) as usize,
        };
    }

    pub fn new_from_world(world: &mut World) -> Box<dyn AssetRequest> {
        let cache = SystemState::<Res<AssetCache>>::new(world).get(world);
        Box::new(RequestContext::new(cache))
    }

    fn expand_buffer(&mut self, b: usize, e: usize) {
        let n = self.buffer.len() + b + self.offset;
        let ps = self.palette.size.element_product() as usize * 4;
        if n > ps {
            panic!("No space {n} out of {ps}")
        }
        self.buffer.append(&mut vec![0; b + e.min(ps - n)]);
    }
}

impl PaletteAllocator for RequestContext {
    fn allocate(&mut self, size: usize) -> usize {
        let s = self.palette.allocate(size);
        return s - self.offset;
    }

    fn get_size(&self) -> PaletteReference {
        return self.palette.get_size();
    }

    fn get_ref_from_idx(&self, idx: usize) -> PaletteReference {
        return self.palette.get_ref_from_idx(idx + self.offset);
    }
}

impl PaletteData for RequestContext {
    fn push_data(&mut self, data: Vec<u8>, start: usize) {
        let end = start + data.len();
        if end > self.buffer.len() {
            self.expand_buffer(end - self.buffer.len(), 300);
        }

        self.buffer.splice(start..end, data);
    }
}

impl AssetRequest for RequestContext {
    fn request_texture(&mut self, texture: String, address: usize) -> () {
        self.references[address] = Some(AssetReference::Texture(texture));
    }

    fn request_palette(&mut self, palette: &[RGBA], address: usize) -> () {
        self.references[address] = Some(AssetReference::Palette(self.add_palette(palette)));
    }

    fn pop(&mut self) -> Vec<Option<AssetReference>> {
        let mut r = Vec::new();
        swap(&mut r, &mut self.references);
        return r;
    }

    fn fill(&mut self, size: usize) -> Result<(), &'static str> {
        if self.references.len() > 0 {
            return Err("Context is not empty");
        }
        for _ in 0..size {
            self.references.push(None)
        }
        return Ok(());
    }

    fn flush(self: Box<Self>, world: &mut World) {
        let (mut cache, mut image) =
            SystemState::<(ResMut<AssetCache>, ResMut<Assets<Image>>)>::new(world).get_mut(world);
        cache.palette.allocator = self.palette;
        image
            .get_mut(&cache.palette.img)
            .as_mut()
            .unwrap()
            .data
            .as_mut()
            .unwrap()
            .splice(self.offset..(self.offset + self.buffer.len()), self.buffer);
    }
}

pub type RequestBuilder = fn(&mut World) -> Box<dyn AssetRequest>;

pub mod batch {
    use bevy::{
        ecs::{entity::Entity, resource::Resource, system::Commands},
        math::primitives::Rectangle,
    };
    use manaengine::rendering::{assetinterface::Asset as GameAsset, cardrender::Assetable};

    use crate::assetmanager::cache::AssetContainer;

    use super::*;

    pub fn render_cards<C: Assetable, A: AssetContainer + Resource>(
        world: &mut World,
        rq_builder: RequestBuilder,
        cards: &Vec<C>,
        card_size: Rectangle,
    ) -> Vec<Entity> {
        let commands = world.commands();
        let entities = generate_bases(commands, cards.len());

        let mut context = rq_builder(world);
        let card_requests = request_assets(cards, context.as_mut());
        let assets = world.resource::<A>();
        let card_info = obtain_assets(assets, card_requests);
        context.flush(world);

        generate_cards(world, card_info, &entities, card_size);

        return entities;
    }

    fn generate_bases(mut commands: Commands, amount: usize) -> Vec<Entity> {
        return (0..amount).map(|_| commands.spawn(()).id()).collect();
    }

    fn request_assets<'a: 'b, 'b, C: Assetable>(
        cards: &'a Vec<C>,
        context: &'b mut dyn AssetRequest,
    ) -> Vec<(&'a C, Vec<AssetReference>)> {
        let mut references = Vec::new();
        for c in cards {
            context.fill(c.get_asset_count()).unwrap();
            c.request_assets(context);
            references.push((c, context.pop().into_iter().map(|r| r.unwrap()).collect()));
        }
        return references;
    }

    fn obtain_assets<T, A: AssetContainer>(
        cache: &A,
        mut requests: Vec<(T, Vec<AssetReference>)>,
    ) -> Vec<(T, Vec<GameAsset>)> {
        let mut assets = Vec::new();
        for (c, ref_list) in requests.drain(..) {
            assets.push((
                c,
                cache
                    .fetch_assets(ref_list)
                    .into_iter()
                    .map(|a| a.unwrap())
                    .collect(),
            ));
        }
        return assets;
    }

    fn generate_cards<C: Assetable>(
        world: &mut World,
        card_info: Vec<(&C, Vec<GameAsset>)>,
        entities: &Vec<Entity>,
        card_size: Rectangle,
    ) {
        for (i, (c, assets)) in card_info.into_iter().enumerate() {
            let base_entity = entities[i].clone();
            c.generate_layers(world, card_size, base_entity, assets);
        }
    }
}

pub fn render_card<C: Assetable, A: AssetContainer + Resource>(
    world: &mut World,
    rq_builder: RequestBuilder,
    card: &C,
    card_size: Rectangle,
) -> Entity {
    let mut commands = world.commands();
    let entity = commands.spawn(()).id();

    let ref_list;
    let mut context = rq_builder(world);

    context.fill(card.get_asset_count()).unwrap();
    card.request_assets(context.as_mut());
    ref_list = context.pop().into_iter().map(|a| a.unwrap()).collect();

    context.flush(world);

    let assets = world.resource::<A>();
    let card_assets = assets.fetch_assets(ref_list);

    let card_assets = card_assets.into_iter().map(|a| a.unwrap()).collect();

    card.generate_layers(world, card_size, entity, card_assets);
    return entity;
}
