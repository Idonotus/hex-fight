use rand::{
	Rng,
	RngCore
};
use std::{cmp::Ordering, marker::PhantomData};
use std::ops::Add;

use crate::engine::colors::{
	Color,
	ColorComparison
};

pub trait IdDeck {
	fn put_card(&mut self, card_id: u64) -> bool;
	fn pop_card(&mut self, card_id: u64) -> bool;

	fn get_max_size(&self) -> u64;
	fn get_size(&self) -> u64;
	fn is_empty(&self) -> bool;
	fn map_available_to_overall(&self, available_id: u64) -> u64;
}

pub trait RandomDraw: IdDeck {
	fn draw_card_random(&mut self, rng: &mut dyn RngCore) -> Option<u64> {
		if self.is_empty() {
			return None;
		}
		let card = rng.random_range(0..self.get_size());
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
	pub fn new(length: u64) -> Self {
		return Self {
			deck: vec![length],
			current_size: length,
			overall_size: length,
			startpresent: true,
		}
	}

	fn attempt_collapse(&mut self, idx: usize) {
		if self.deck[idx] != 0 {return;}
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

	fn set_state(&mut self, card: u64, state: bool) -> bool {
		let mut dist_tracker = card;
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
	fn get_max_size(&self) -> u64 {
		return self.overall_size;
	}

	fn get_size(&self) -> u64 {
		return self.current_size;
	}

	fn is_empty(&self) -> bool {
		return self.current_size <= 0;
	}

	fn put_card(&mut self, card_id: u64) -> bool {
		if card_id > self.overall_size {
			panic!("Card doesn't belong to deck");
		}
		let res = self.set_state(card_id, true);
		if res {
			self.current_size += 1;
		}
		return res;
	}

	fn pop_card(&mut self, card_id: u64) -> bool {
		if card_id > self.overall_size {
			panic!("Card doesn't belong to deck");
		}
		let res = self.set_state(card_id, false);
		if res {
			self.current_size -= 1;
		}
		return res;
	}

	fn map_available_to_overall(&self, available_id: u64) -> u64 {
		let mut total = 0u64;
		let mut dist_tracker = available_id;
		let mut exist_tracker = self.startpresent;
		for slice in self.deck.iter() {
			if *slice > dist_tracker {
				total += dist_tracker;
				break;
			}
			total += slice;
			dist_tracker -= if exist_tracker {*slice} else {0};
			exist_tracker = !exist_tracker;
		}
		return  total;
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
		if self.get_value() == head.get_value() {return true;}
		if None == head.get_color() {return true;}
		return color_comparason(self.get_color().unwrap(), head.get_color().unwrap());
	}
	fn can_stack_onto(&self, base: &dyn Stacks, color_comparason: &ColorComparison) -> bool {
		if self.get_value() == base.get_value() {return true;}
		if None == base.get_color() {return true;}
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

pub fn does_stack(base: &dyn Stacks, head: &dyn Stacks, color_comparason: &ColorComparison) -> bool {
	match base.get_stacking_priority().cmp(&head.get_stacking_priority()) {
		Ordering::Less | Ordering::Equal => {base.can_get_stacked(head, color_comparason)},
		Ordering::Greater => {head.can_stack_onto(base, color_comparason)},
	}
}

pub(crate) trait BaseBand {
	fn get_band_size(&self) -> u64;
}

pub(crate) trait AssignedBand<'a, C>: BaseBand {
	fn generate_card(&mut self, c_id: u64) -> C;
}

impl<'a, C> BaseBand for Box<dyn AssignedBand<'a, C> + 'a> {
	fn get_band_size(&self) -> u64 {
		self.as_ref().get_band_size()
	}
}

impl<'a, C> AssignedBand<'a, C> for Box<dyn AssignedBand<'a, C> + 'a> {
	fn generate_card(&mut self, c_id: u64) -> C {
		self.as_mut().generate_card(c_id)
	}
}

pub(crate) struct BandSet<'a, Band, C>(Vec<Band>, PhantomData<&'a C>) where Band: AssignedBand<'a, C>;

impl<'a, Band, T> BandSet<'a, Band, T>
where Band: AssignedBand<'a, T> {
	pub fn new(set: Vec<Band>) -> Self {
		Self(set, PhantomData)
	}

	pub fn iter(&self) -> std::slice::Iter<'_, Band> {
		self.0.iter()
	}
}

impl<'a, T, Band> Add<BandSet<'a, Band, T>> for BandSet<'a, Band, T>
where Band: AssignedBand<'a, T> {
	type Output = BandSet<'a, Band, T>;
	fn add(mut self, mut rhs: BandSet<'a, Band, T>) -> Self::Output {
		self.0.append(&mut rhs.0);
		self
	}
}

impl<'a, Band: AssignedBand<'a, T>, T> BaseBand for BandSet<'a, Band, T> {
	fn get_band_size(&self) -> u64 {
		self.0.iter().map(|b| {b.get_band_size()}).sum()
	}
}

impl<'a, T, Band> AssignedBand<'a, T> for BandSet<'a, Band, T>
where Band: AssignedBand<'a, T> {
	fn generate_card(&mut self, c_id: u64) -> T {
		let mut rest = c_id;
		for band in &mut self.0 {
			let size = band.get_band_size();
			if rest < size {
				return band.generate_card(rest);
			}
			rest -= size;
		}
		panic!("Card is out of set")
	}
}