use std::ops::Add;

pub(crate) trait AssignedBand<'a, C> {
	fn get_band_size(&self) -> u64;
	fn generate_card(&mut self, c_id: u64) -> C;
}

pub(crate) struct BandSet<'a, T>(Vec<Box<dyn AssignedBand<'a, T>>>);

impl<'a, T> BandSet<'a, T> {
	pub fn new(set: Vec<Box<dyn AssignedBand<'a, T>>>) -> Self {
		Self(set)
	}
}

impl<'a, T> Add<BandSet<'a, T>> for BandSet<'a, T> {
	type Output = BandSet<'a, T>;
	fn add(mut self, mut rhs: BandSet<'a, T>) -> Self::Output {
		self.0.append(&mut rhs.0);
		self
	}
}

impl<'a, T> AssignedBand<'a, T> for BandSet<'a, T> {
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

	fn get_band_size(&self) -> u64 {
		self.0.iter().map(|b| {b.get_band_size()}).sum()
	}
}