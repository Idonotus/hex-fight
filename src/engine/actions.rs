
enum ContextItem {
	Player(usize),
	CardReference(u64),
	Card(Card),
	Number(i64),
	Decimal(f32),
	Game(Game),
}

impl ContextItem {
	fn get_predicate(&self) -> ContextPredicate {
		match self {
			ContextItem::Card(_) => ContextPredicate::Card,
			ContextItem::CardReference(_) => ContextPredicate::CardReference,
			ContextItem::Decimal(_) => ContextPredicate::Decimal,
			ContextItem::Game(_) => ContextPredicate::Game,
			ContextItem::Player(_) => ContextPredicate::Player,
			ContextItem::Number(_) => ContextPredicate::Number,
		}
	}
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum ContextPredicate {
	Player,
	CardReference,
	Card,
	Number,
	Decimal,
	Game,
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
	fn allocate(&mut self, defaults: Vec<ContextItem>) -> Vec<usize> {
		todo!()
	}

	fn drop_level(&mut self) {
		todo!()
	}
	
	fn reference_item(&self, reference: usize) -> ContextItem {
		todo!()
	}

	fn borrow_item(&mut self, reference: usize) -> ContextItem {
		todo!()
	}

	fn return_item(&mut self, reference: usize, item: ContextItem) -> () {
		todo!()
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

trait Labelled {
	fn get_label(&self) -> &str;
}

trait Action {

}