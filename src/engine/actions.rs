use std::mem::{ replace };

enum ContextItem {
	Player(usize),
	CardReference(u64),
//	Card(Card),
	Number(i64),
	Decimal(f32),
//	Game(Game),
}

impl ContextItem {
	fn get_predicate(&self) -> ContextPredicate {
		match self {
//			ContextItem::Card(_) => ContextPredicate::Card,
			ContextItem::CardReference(_) => ContextPredicate::CardReference,
			ContextItem::Decimal(_) => ContextPredicate::Decimal,
//			ContextItem::Game(_) => ContextPredicate::Game,
			ContextItem::Player(_) => ContextPredicate::Player,
			ContextItem::Number(_) => ContextPredicate::Number,
		}
	}
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum ContextPredicate {
	Player,
	CardReference,
//	Card,
	Number,
	Decimal,
//	Game,
}

fn predicate_legal(a: &[ContextPredicate], b: &[ContextPredicate]) -> bool {
	if a.len() != b.len() {return false;}

	for i in 0..a.len() {
		if a[i] != b[i] {return false;}
	}

	return true;
}

struct Response {
	queue: Vec<Box<dyn Action>>,
	context: ResponseContext,
	
}

struct ResponseContext {
	context_items: Vec<Option<ContextItem>>,
	predicates: Vec<ContextPredicate>,
	levels: Vec<usize>,
}



impl ResponseContext {
	fn allocate_from_vec(&mut self, defaults: Vec<ContextItem>) -> Vec<usize> {
		self.predicates.append(&mut defaults.iter().map(ContextItem::get_predicate).collect());
		let prev = self.context_items.len();
		self.context_items.append(&mut defaults.into_iter().map(|c| {Some(c)}).collect());
		let post = self.context_items.len();
		let references: Vec<usize> = (prev..post).collect();
		self.levels.push(references.len());
		references
	}

	fn drop_level(&mut self) -> () {
		let i = self.levels.pop().unwrap();
		for _ in 0..i {
			self.context_items.pop();
			self.predicates.pop();
		}
	}
	
	fn reference_item(&self, reference: usize) -> &ContextItem {
		&self.context_items[reference].as_ref().unwrap()
	}

	fn borrow_item(&mut self, reference: usize) -> Result<ContextItem, &str> {
		let item = replace(&mut self.context_items[reference], None);
		match item {
			Some(context) => Ok(context),
			None => Err("Context item has already been borrowed")
		}
	}

	fn return_item(&mut self, reference: usize, item: ContextItem) -> () {
		replace(&mut self.context_items[reference], Some(item));
	}
}

impl Response {
	fn append(&mut self, actions: Vec<&dyn Action>) {
		todo!()
	}

	fn step(&mut self) {
		todo!()
	}
}

trait Action {

}

enum Display {
	ShowHand(u8),
	ObfuscateHand(u8),

}

enum Prompt {
	PickColor,
	PickCardsFromHand(u8, u8),
	PickNumeral,
	PickPlayer,
	Approval,
}