use rand::{Rng, RngCore};
use std::iter::Sum;
use std::ops::{Add, Deref};
use std::{cmp::Ordering, marker::PhantomData};

use crate::engine::colors::{Color, ColorComparison};

// Macro?

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DeckId(pub u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DeckCapacity(pub u64);

impl Sum for DeckCapacity {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        DeckCapacity(iter.map(|c| *c).sum())
    }
}

impl Add for DeckCapacity {
    type Output = DeckCapacity;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.0 += rhs.0;
        self
    }
}

impl PartialEq<DeckId> for DeckCapacity {
    fn eq(&self, other: &DeckId) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<DeckId> for DeckCapacity {
    fn partial_cmp(&self, other: &DeckId) -> Option<Ordering> {
        if self.0 > other.0 {
            return Some(Ordering::Greater);
        }
        if self.0 < other.0 {
            return Some(Ordering::Less);
        }
        return Some(Ordering::Equal);
    }
}

impl Deref for DeckId {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for DeckCapacity {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct VirtualDeckId(pub u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct DeckSize(pub u64);

impl PartialEq<VirtualDeckId> for DeckSize {
    fn eq(&self, other: &VirtualDeckId) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<VirtualDeckId> for DeckSize {
    fn partial_cmp(&self, other: &VirtualDeckId) -> Option<Ordering> {
        if self.0 > other.0 {
            return Some(Ordering::Greater);
        }
        if self.0 < other.0 {
            return Some(Ordering::Less);
        }
        return Some(Ordering::Equal);
    }
}

impl Deref for VirtualDeckId {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for DeckSize {
    type Target = u64;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait IdDeck {
    fn put_card(&mut self, card_id: DeckId) -> bool;
    fn pop_card(&mut self, card_id: DeckId) -> bool;

    fn get_max_size(&self) -> DeckCapacity;
    fn get_size(&self) -> DeckSize;
    fn is_empty(&self) -> bool;
    fn map_available_to_overall(&self, available_id: VirtualDeckId) -> DeckId;
}

pub trait RandomDraw: IdDeck {
    fn draw_card_random(&mut self, rng: &mut dyn RngCore) -> Option<DeckId> {
        if self.is_empty() {
            return None;
        }
        let card = VirtualDeckId(rng.random_range(0..*self.get_size()));
        let card_id = self.map_available_to_overall(card);
        if !self.pop_card(card_id) {
            panic!("Mapping provided card that was already drawn")
        }

        return Some(card_id);
    }
}

pub struct RLEDeck {
    deck: Vec<u64>,
    current_size: u64,
    overall_size: u64,
    startpresent: bool,
}

impl RLEDeck {
    pub fn new(length: DeckCapacity) -> Self {
        let len = length.0;
        return Self {
            deck: vec![len],
            current_size: len,
            overall_size: len,
            startpresent: true,
        };
    }

    fn attempt_collapse(&mut self, idx: usize) {
        if self.deck[idx] != 0 {
            return;
        }
        let l = self.deck.len();
        if idx + 1 == l {
            self.deck.pop();
            return;
        }
        if idx == 0 {
            self.deck.remove(0);
            self.startpresent = !self.startpresent;
            return;
        }
        self.deck.remove(idx);
        let extra = self.deck.remove(idx);
        self.deck[idx - 1] += extra;
    }

    fn set_state(&mut self, card: DeckId, state: bool) -> bool {
        let mut dist_tracker = card.0;
        let mut exist_tracker = self.startpresent;
        let mut latest_i = 0usize;
        for (i, slice) in self.deck.iter().enumerate() {
            latest_i = i;
            if *slice > dist_tracker {
                break;
            }
            exist_tracker = !exist_tracker;
            dist_tracker -= slice;
        }
        if exist_tracker == state {
            return false;
        }
        self.flip_in_block(latest_i, dist_tracker, 1);
        return true;
    }

    fn flip_in_block(&mut self, index: usize, offset: u64, size: u64) {
        if (self.deck[index] - offset) < size {
            panic!("Deck doesn't have continuous block to flip")
        }
        let remaining_size = self.deck[index] - offset - size;
        if index == 0 {
            if remaining_size == 0 && offset == 0 {
                self.startpresent = !self.startpresent;
                self.deck.remove(0);
                self.deck[0] += size;
                return;
            } else if offset == 0 {
                self.deck.insert(0, size);
                self.deck[1] -= size;
                self.startpresent = !self.startpresent;
                return;
            }
        }
        if index == self.deck.len() - 1 {
            if remaining_size == 0 && offset == 0 {
                self.deck[index - 1] += size;
                return;
            } else if remaining_size == 0 {
                self.deck.push(size);
                return;
            }
        }
        if remaining_size == 0 && offset == 0 {
            self.deck[index - 1] += self.deck.remove(index);
            self.deck[index - 1] += self.deck.remove(index);
            return;
        }
        if offset == 0 {
            self.deck[index - 1] += size;
            self.deck[index] -= size;
            return;
        }
        if remaining_size == 0 {
            self.deck[index] -= size;
            self.deck[index + 1] += size;
            return;
        }
        self.deck[index] = offset;
        self.deck.insert(index + 1, remaining_size);
        self.deck.insert(index + 1, size);
        return;
    }
}

impl IdDeck for RLEDeck {
    fn get_max_size(&self) -> DeckCapacity {
        return DeckCapacity(self.overall_size);
    }

    fn get_size(&self) -> DeckSize {
        return DeckSize(self.current_size);
    }

    fn is_empty(&self) -> bool {
        return self.current_size <= 0;
    }

    fn put_card(&mut self, card_id: DeckId) -> bool {
        if card_id.0 > self.overall_size {
            panic!("Card doesn't belong to deck");
        }
        let res = self.set_state(card_id, true);
        if res {
            self.current_size += 1;
        }
        return res;
    }

    fn pop_card(&mut self, card_id: DeckId) -> bool {
        if card_id.0 > self.overall_size {
            panic!("Card doesn't belong to deck");
        }
        let res = self.set_state(card_id, false);
        if res {
            self.current_size -= 1;
        }
        return res;
    }

    fn map_available_to_overall(&self, available_id: VirtualDeckId) -> DeckId {
        let mut total = DeckId(0);
        let mut dist_tracker = available_id;
        let mut exist_tracker = !self.startpresent;
        for slice in self.deck.iter() {
            exist_tracker = !exist_tracker;
            if !exist_tracker {
                total.0 += slice;
                continue;
            }
            if *slice > dist_tracker.0 {
                total.0 += dist_tracker.0;
                break;
            }
            total.0 += slice;
            dist_tracker.0 -= *slice;
        }
        return total;
    }
}

impl RandomDraw for RLEDeck {}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CardValue {
    Special(usize),
    Numeral(u8),
}

pub trait Stacks {
    fn get_stacking_priority(&self) -> i16;
    fn can_get_stacked(&self, head: &dyn Stacks, color_comparason: &ColorComparison) -> bool {
        if self.get_value() == head.get_value() {
            return true;
        }
        if None == head.get_color() {
            return true;
        }
        return color_comparason(self.get_color().unwrap(), head.get_color().unwrap());
    }
    fn can_stack_onto(&self, base: &dyn Stacks, color_comparason: &ColorComparison) -> bool {
        if self.get_value() == base.get_value() {
            return true;
        }
        if None == base.get_color() {
            return true;
        }
        return color_comparason(self.get_color().unwrap(), base.get_color().unwrap());
    }
    fn get_color(&self) -> Option<Color>;
    fn get_value(&self) -> CardValue;
}

impl Stacks for Box<dyn Stacks> {
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

pub fn does_stack(
    base: &dyn Stacks,
    head: &dyn Stacks,
    color_comparason: &ColorComparison,
) -> bool {
    match base
        .get_stacking_priority()
        .cmp(&head.get_stacking_priority())
    {
        Ordering::Less | Ordering::Equal => base.can_get_stacked(head, color_comparason),
        Ordering::Greater => head.can_stack_onto(base, color_comparason),
    }
}

pub trait BaseBand {
    fn get_band_size(&self) -> DeckCapacity;
}

pub trait AssignedBand<'a, C>: BaseBand {
    fn generate_card(&mut self, c_id: DeckId) -> C;
}

impl<'a, C> BaseBand for Box<dyn AssignedBand<'a, C> + 'a> {
    fn get_band_size(&self) -> DeckCapacity {
        self.as_ref().get_band_size()
    }
}

impl<'a, C> AssignedBand<'a, C> for Box<dyn AssignedBand<'a, C> + 'a> {
    fn generate_card(&mut self, c_id: DeckId) -> C {
        self.as_mut().generate_card(c_id)
    }
}

pub(crate) struct BandSet<'a, Band, C>(Vec<Band>, PhantomData<&'a C>)
where
    Band: AssignedBand<'a, C>;

impl<'a, Band, T> BandSet<'a, Band, T>
where
    Band: AssignedBand<'a, T>,
{
    pub fn new(set: Vec<Band>) -> Self {
        Self(set, PhantomData)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Band> {
        self.0.iter()
    }
}

impl<'a, T, Band> Add<BandSet<'a, Band, T>> for BandSet<'a, Band, T>
where
    Band: AssignedBand<'a, T>,
{
    type Output = BandSet<'a, Band, T>;
    fn add(mut self, mut rhs: BandSet<'a, Band, T>) -> Self::Output {
        self.0.append(&mut rhs.0);
        self
    }
}

impl<'a, Band: AssignedBand<'a, T>, T> BaseBand for BandSet<'a, Band, T> {
    fn get_band_size(&self) -> DeckCapacity {
        self.0.iter().map(|b| b.get_band_size()).sum()
    }
}

impl<'a, T, Band> AssignedBand<'a, T> for BandSet<'a, Band, T>
where
    Band: AssignedBand<'a, T>,
{
    fn generate_card(&mut self, c_id: DeckId) -> T {
        let mut rest = c_id;
        for band in &mut self.0 {
            let size = band.get_band_size();
            if size > rest {
                return band.generate_card(rest);
            }
            rest.0 -= size.0;
        }
        panic!("Card is out of set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_deck() -> RLEDeck {
        RLEDeck::new(DeckCapacity(100))
    }

    #[test]
    fn draw_pop() {
        let mut deck = base_deck();
        for idx in 0..*deck.get_size() {
            assert!(deck.pop_card(DeckId(idx)));
            assert!(!deck.pop_card(DeckId(idx)));
        }
    }

    #[test]
    fn rle_check() {
        let mut deck = base_deck();
        deck.pop_card(DeckId(0));
        assert_eq!(deck.deck.len(), 2);
        assert!(!deck.startpresent);
        deck.put_card(DeckId(0));
        assert!(deck.startpresent);
        deck.pop_card(DeckId(3));
        assert_eq!(deck.deck[0], 3);
        assert_eq!(deck.deck[1], 1);
        assert_eq!(deck.deck[2], 96);
    }

    #[test]
    fn simple_mapping() {
        let deck = base_deck();
        for idx in 0..100u64 {
            let r = deck.map_available_to_overall(VirtualDeckId(idx));
            println!("{:?}", deck.deck);
            assert_eq!(r, DeckId(idx));
        }
    }

    #[test]
    fn extended_mapping() {
        let mut deck = base_deck();
        for idx in 0..100u64 {
            let r = deck.map_available_to_overall(VirtualDeckId(0));
            println!("{:?} {r:?}", deck.deck);
            assert_eq!(r, DeckId(idx));
            assert!(deck.pop_card(r));
        }
        assert!(deck.is_empty());
    }
}
