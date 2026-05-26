use std::cmp::Ordering;

use bevy::{ecs::resource::Resource, prelude::*};

#[derive(Debug, PartialEq, Eq)]
enum Cap {
    Fixed(usize),
    Uncapped,
}

impl PartialOrd for Cap {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if other == self {
            return Some(Ordering::Equal);
        }
        match self {
            Cap::Fixed(a) => {
                let Cap::Fixed(b) = other else {
                    return Some(Ordering::Less);
                };
                if a > b {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            Cap::Uncapped => Some(Ordering::Greater),
        }
    }
}

#[derive(Resource)]
pub struct Selections {
    held_cards: Vec<Entity>,
    max_capacity: Option<usize>,
}

impl Selections {
    pub fn new(capacity: Option<usize>) -> Self {
        Self {
            held_cards: match capacity {
                Some(c) => Vec::with_capacity(c),
                None => Vec::new(),
            },
            max_capacity: capacity,
        }
    }

    pub fn get_capacity(&self) -> &Option<usize> {
        &self.max_capacity
    }

    pub fn set_capacity(&mut self, cap: Option<usize>) {
        if self.held_cards.len() != 0 {
            panic!()
        }
        if let Some(capacity) = cap {
            self.held_cards = Vec::with_capacity(capacity);
        }
        self.max_capacity = cap;
    }

    pub fn get_held_cards(&self) -> &[Entity] {
        &self.held_cards
    }

    pub fn push(&mut self, card: Entity) {
        if let Some(c) = self.max_capacity {
            if self.held_cards.len() == c {
                panic!()
            }
        }
        self.held_cards.push(card)
    }

    pub fn find(&self, card: Entity) -> Option<usize> {
        for (i, c) in self.held_cards.iter().enumerate() {
            if *c == card {
                return Some(i);
            }
        }
        return None;
    }

    pub fn remove(&mut self, idx: usize) {
        self.held_cards.remove(idx);
    }

    pub fn is_full(&self) -> bool {
        match self.max_capacity {
            None => false,
            Some(s) => s <= self.held_cards.len(),
        }
    }
}
