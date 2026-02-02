use crate::engine::cards::Card;

trait AssignedBand {
	fn get_band_size(&self) -> u64;
	fn generate_card(&mut self, c_id: u64) -> Card;
}

struct CardSet {
	total: Vec<Box<dyn AssignedBand>>
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