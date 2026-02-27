mod assets;

use std::marker::PhantomData;

use crate::engine::{
	cards::{
		CardValue,
		Card,
		Stacks,
		AssignedBand
	},
	colors::Color,
};

pub struct AllColorBand {
	numeral: u8
}

impl AllColorBand {
	pub fn new(n: u8) -> Self {
		Self { numeral: n }
	}
}

impl<'a> AssignedBand<'a, Card<'a>> for AllColorBand {
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

impl Stacks for SimpleCard {
	fn get_value(&self) -> CardValue {
		return self.value;
	}

	fn get_color(&self) -> Option<Color> {
		return Some(self.color);
	}

	fn get_stacking_priority(&self) -> i16 {
		0
	}
}

pub struct SimpleCard {
	color: Color,
	value: CardValue,
}

impl SimpleCard {
	pub fn new(color: Color, value: u8) -> Self {
		Self { color, value: CardValue::Numeral(value) }
	}
}

pub struct ChooseColorCard {
	color: Option<Color>,
	value: CardValue,
}

struct PluralBand<'a, Band, C>(Band, u64, PhantomData<&'a C>)
where Band: AssignedBand<'a, C>;

impl<'a, Band, C> PluralBand<'a, Band, C>
where Band: AssignedBand<'a, C> {
	fn new(band: Band, amount: u64) -> Self {
		Self(band, amount, PhantomData)
	}
}

impl<'a, Band, C> AssignedBand<'a, C> for PluralBand<'a, Band, C>
where Band: AssignedBand<'a, C> {
	fn generate_card(&mut self, c_id: u64) -> C {
		self.0.generate_card(c_id % self.1)	
	}

	fn get_band_size(&self) -> u64 {
		self.0.get_band_size() * self.1
	}
}