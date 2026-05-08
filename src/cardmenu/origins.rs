use bevy::{
    ecs::{entity::Entity, resource::Resource, system::Commands},
    platform::collections::HashMap,
};

use crate::engine::prelude::*;

#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Eq)]
enum CardOrigin {
    Hand(PlayerId, usize),
    Deck(CardId),
    TopCard,
}

#[derive(Resource)]
pub struct CardUIOrigins {
    origins: HashMap<Entity, CardOrigin>,
    integrity_map: HashMap<CardOrigin, Vec<Entity>>,
}

impl CardUIOrigins {
    pub fn new() -> Self {
        Self {
            origins: HashMap::new(),
            integrity_map: HashMap::new(),
        }
    }

    pub fn register_card(&mut self, entity: Entity, origin: CardOrigin) {
        self.origins.insert(entity, origin);
        if self.integrity_map.contains_key(&origin) {
            self.integrity_map
                .get_mut(&origin)
                .expect("Contained key but doesn't have data")
                .push(entity);
        } else {
            self.integrity_map.insert(origin, vec![entity]);
        }
    }

    pub fn deregister_card(&mut self, entity: &Entity) -> Option<CardOrigin> {
        let origin = self.origins.remove(entity);
        match origin {
            Some(o) => {
                let es = self
                    .integrity_map
                    .get_mut(&o)
                    .expect("Origin map does not match integrity map");
                if es.len() <= 1 {
                    self.integrity_map.remove(&o);
                } else {
                    let mut f = 0;
                    for (i, e) in es.iter().enumerate() {
                        if e == entity {
                            f = i;
                            break;
                        }
                    }
                    es.remove(f);
                }
            }
            _ => {}
        }
        return origin;
    }

    pub fn get_origin(&mut self, entity: &Entity) -> Option<CardOrigin> {
        self.origins.get(entity).copied()
    }

    pub fn delete_origin(&mut self, origin: &CardOrigin) -> Vec<Entity> {
        match self.integrity_map.remove(origin) {
            Some(entities) => {
                for e in entities.iter() {
                    self.origins.remove_entry(e);
                }
                return entities;
            }
            None => {
                return Vec::new();
            }
        }
    }
}
