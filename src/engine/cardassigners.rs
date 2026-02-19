use crate::engine::
	{
		cards::{
			Card,
			SimpleCard
		},
		colors::Color,
	};
use std::ops::Add;

pub trait AssignedBand<'a> {
	fn get_band_size(&self) -> u64;
	fn generate_card(&mut self, c_id: u64) -> Card<'a>;
}

pub struct CardSet<'a> {
	total: Vec<Box<dyn AssignedBand<'a>>>
}

impl<'a> CardSet<'a> {
	pub fn new(set: Vec<Box<dyn AssignedBand<'a>>>) -> Self {
		Self { total: set }
	}
}

impl<'a> Add<CardSet<'a>> for CardSet<'a> {
	type Output = CardSet<'a>;
	fn add(mut self, mut rhs: CardSet<'a>) -> Self::Output {
		self.total.append(&mut rhs.total);
		self
	}
}

impl<'a> AssignedBand<'a> for CardSet<'a> {
	fn generate_card(&mut self, c_id: u64) -> Card<'a> {
		let mut rest = c_id;
		for band in &mut self.total {
			let size = band.get_band_size();
			if rest < size {
				return band.generate_card(rest);
			}
			rest -= size;
		}
		panic!("Card is out of set")
	}

	fn get_band_size(&self) -> u64 {
		self.total.iter().map(|b| {b.get_band_size()}).sum()
	}
}

pub struct AllColorBand {
	numeral: u8
}

impl AllColorBand {
	pub fn new(n: u8) -> Self {
		Self { numeral: n }
	}
}

impl<'a> AssignedBand<'a> for AllColorBand {
	fn generate_card(&mut self, c_id: u64) -> Card<'a> {
		let (c_id, r) = (c_id / 256, (c_id % 256).try_into().unwrap());
		let (c_id, g) = (c_id / 256, (c_id % 256).try_into().unwrap());
		let (value, b) = ((c_id / 256).try_into().unwrap(), (c_id % 256).try_into().unwrap());
		
		
		return Box::new(SimpleCard::new(
			Color {
				r,
				g,
				b
			},
			value
		));
	}

	fn get_band_size(&self) -> u64 {
		let i: u64 = self.numeral.into();
		return 0x1000000u64 * i;
	}
}