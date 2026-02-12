use std::mem::{ replace };
use crate::engine::colors::Color;

trait Predicate<T> {
	fn get_predicate(&self) -> T;
}

pub enum ContextItem {
	Player(usize),
	CardReference(u64),
	Color(Color),
	Number(i64),
	Decimal(f32),
	Boolean(bool),
//	Card(Card),
//	Game(Game),
}

impl Predicate<ContextPredicate> for ContextItem {
	fn get_predicate(&self) -> ContextPredicate {
		match self {
			ContextItem::Color(_) => ContextPredicate::Color,
			ContextItem::CardReference(_) => ContextPredicate::CardReference,
			ContextItem::Decimal(_) => ContextPredicate::Decimal,
			ContextItem::Player(_) => ContextPredicate::Player,
			ContextItem::Number(_) => ContextPredicate::Number,
			ContextItem::Boolean(_) => ContextPredicate::Boolean,
//			ContextItem::Game(_) => ContextPredicate::Game,
//			ContextItem::Card(_) => ContextPredicate::Card,
		}
	}
}

struct BorrowedItem {
	reference: usize,
	context: ContextItem,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum ContextPredicate {
	Player,
	Color,
	CardReference,
	Boolean,
	Number,
	Decimal,
//	Card,
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

pub trait Action {
	fn get_predicate(&self) -> Vec<ContextPredicate>;
	fn get_required_references(&self) -> Vec<usize>;
	fn run_action(&mut self, parameters: Vec<ContextItem>) -> Vec<ContextItem>;
}

pub enum Display {
	ShowHand(u8),
	ObfuscateHand(u8),
}

pub enum Prompt {
	PickColor {amount: usize},
	PickCardsFromHand {player: usize, amount: usize},
	PickNumeral {amount: usize},
	PickPlayer {amount: usize},
	Approval,
}

impl Predicate<ContextPredicate> for Prompt {
	fn get_predicate(&self) -> ContextPredicate {
		match self {
			Prompt::PickColor { amount: _ } => ContextPredicate::Color,
			Prompt::Approval => ContextPredicate::Boolean,
			Prompt::PickPlayer { amount: _ } => ContextPredicate::Player,
			Prompt::PickNumeral { amount: _ } => ContextPredicate::Number,
			Prompt::PickCardsFromHand { player: _, amount: _ } => ContextPredicate::CardReference,
		}
	}
}

pub enum Interaction<'a> {
	UserPrompt {
		player: usize,
		prompt: Prompt,
		id: &'a str
	},
	UserDisplay {
		player: Vec<usize>,
		display: Display,
		id: &'a str
	},
	UserPlays {
		player: usize,
		prompts: Vec<(&'a str, Prompt)>
	}
}