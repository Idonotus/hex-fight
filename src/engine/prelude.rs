#[derive(PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Copy)]
pub struct PlayerId(pub usize);

impl Deref for PlayerId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use std::ops::Deref;

pub use super::cards::{
    AssignedBand, BandSet, BaseBand, DeckCapacity, DeckId as CardId, DeckSize, Stacks,
};
