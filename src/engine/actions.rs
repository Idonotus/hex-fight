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

impl Into<bool> for ContextItem {
	fn into(self) -> bool {
		match self {
			ContextItem::Boolean(v) => v,
			_ => panic!()
		}
	}
}
impl Into<Color> for ContextItem {
	fn into(self) -> Color {
		match self {
			ContextItem::Color(v) => v,
			_ => panic!()
		}
	}
}
impl Into<u64> for ContextItem {
	fn into(self) -> u64 {
		match self {
			ContextItem::CardReference(v) => v,
			_ => panic!()
		}
	}
}
impl Into<i64> for ContextItem {
	fn into(self) -> i64 {
		match self {
			ContextItem::Number(v) => v,
			_ => panic!()
		}
	}
}
impl Into<f32> for ContextItem {
	fn into(self) -> f32 {
		match self {
			ContextItem::Decimal(v) => v,
			_ => panic!()
		}
	}
}
impl Into<usize> for ContextItem {
	fn into(self) -> usize {
		match self {
			ContextItem::Player(v) => v,
			_ => panic!()
		}
	}
}

struct ItemReference {
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

struct Response<'a> {
	queue: Vec<Box<dyn Action<'a> + 'a>>,
	context: ResponseContext,
	level_points: Vec<usize>,
}

struct ResponseContext {
	context_items: Vec<Option<ContextItem>>,
	predicates: Vec<ContextPredicate>,
	levels: Vec<usize>,
}

impl ResponseContext {
	fn get_predicates(&self, params: &Vec<usize>) -> Vec<ContextPredicate> {
		params.iter().map(|i| {self.predicates[*i]}).collect()
	}

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

impl<'a> Response<'a> {
	fn append(&mut self, actions: Vec<&'a dyn Action<'a>>) {
		todo!()
	}

	fn step(&mut self) {
		let mut action = self.queue.pop().unwrap();

		let cur_len = self.queue.len();
		for p in self.level_points.iter().rev() {
			let point = *p;
			if point <= cur_len {
				break;
			}
			self.context.drop_level();
		}

		let predicates = action.get_predicate();
		let params = action.get_required_references();
		if !predicate_legal(&predicates, &self.context.get_predicates(&params)) {
			return;
		}
		let mut contexts: Vec<ItemReference> = Vec::with_capacity(predicates.len());
		for i in params.iter() {
			contexts.push(ItemReference {
				reference: *i,
				context: self.context.borrow_item(*i).unwrap()
			});
		}
		let mut proc = action.run_action(contexts);
		
		for (idx, reference) in params.into_iter().enumerate().rev() {
			let item = proc.returned_context.pop().unwrap();
			if predicates[idx] != item.get_predicate() {
				panic!()
			}
			self.context.return_item(reference, item);
		}
		
		let cur_len = self.queue.len();
		let need_context_level = proc.further_processing.len() != 0;
		self.append(proc.further_processing);

		if !need_context_level {
			return;
		} if proc.additional_context.len() != 0 {
			return;
		}

		self.level_points.push(cur_len);
		self.context.allocate_from_vec(proc.additional_context);
	}
}

pub struct ActionResult<'a> {
	returned_context: Vec<ContextItem>,
	additional_context: Vec<ContextItem>,
	further_processing: Vec<&'a dyn Action<'a>>,
}

pub trait Action<'a> {
	fn get_predicate(&self) -> Vec<ContextPredicate>;
	fn get_required_references(&self) -> Vec<usize>;
	fn run_action(self: Box<Self>, parameters: Vec<ItemReference>) -> ActionResult<'a>;
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