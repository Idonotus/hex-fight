use crate::engine::
	{
		cards::{
			Card,
			SimpleCard
		},
		colors::Color,
	};

pub trait AssignedBand {
	fn get_band_size(&self) -> u64;
	fn generate_card(&mut self, c_id: u64) -> Card;
}

pub struct CardSet {
	total: Vec<Box<dyn AssignedBand>>
}

pub struct AllColorBand {
	numeral: u8
}

impl AllColorBand {
	pub fn new(n: u8) -> Self {
		Self { numeral: n }
	}
}

impl AssignedBand for AllColorBand {
	fn generate_card(&mut self, c_id: u64) -> Card {
		let (c_id, r) = (c_id / 256, (c_id % 256).try_into().unwrap());
		let (c_id, g) = (c_id / 256, (c_id % 256).try_into().unwrap());
		let (value, b) = ((c_id / 256).try_into().unwrap(), (c_id % 256).try_into().unwrap());
		
		Box::new(
			SimpleCard::new(
				Color {
					r,
					g,
					b
				},
				value
			)
		)
	}

	fn get_band_size(&self) -> u64 {
		let i: u64 = self.numeral.into();
		return 0x1000000u64 * i;
	}
}

impl CardSet {
	pub fn new(set: Vec<Box<dyn AssignedBand>>) -> Self {
		Self { total: set }
	}
}

impl AssignedBand for CardSet {
	fn generate_card(&mut self, c_id: u64) -> Card {
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