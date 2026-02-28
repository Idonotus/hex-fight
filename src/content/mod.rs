mod assets;

use std::marker::PhantomData;

use assets::{
	AssetBand,
	Assetable,
	AssetReference,
	CardAssets
};

use crate::engine::{
	cards::{
		CardValue,
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

trait Card: Stacks + Assetable {}

impl<'a> AssignedBand<'a, Box<dyn Card + 'a>> for AllColorBand {
	fn generate_card(&mut self, c_id: u64) -> Box<dyn Card + 'a>> {
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

pub struct SimpleCard {
	color: Color,
	value: CardValue,
}

impl SimpleCard {
	pub fn new(color: Color, value: u8) -> Self {
		Self { color, value: CardValue::Numeral(value) }
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

impl Assetable for SimpleCard {
	fn get_assets(&self) -> CardAssets {
		CardAssets {
			name: format!("{:?} of {}", self.get_value(), self.get_color().unwrap()).as_str(),
			description: "A normal card"
		}
	}
}

impl Card for SimpleCard {}

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
		self.0.generate_card(c_id / self.1)	
	}

	fn get_band_size(&self) -> u64 {
		self.0.get_band_size() * self.1
	}
}