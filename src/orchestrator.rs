
trait Card: Stacks + Assetable {}
impl<'a> Assetable for Box<dyn Card + 'a> {
	fn get_assets(&self) -> CardAssets {
		self.as_ref().get_assets()
	}
}
impl<'a> Stacks for Box<dyn Card + 'a> {
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
impl<'a> Card for Box<dyn Card + 'a> {}

fn main() {
	
}