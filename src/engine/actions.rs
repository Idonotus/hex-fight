struct Response {
	queue: Vec<dyn Action>
}

struct ResponseContext {

}

struct ContextItem {

}

impl ResponseContext {
	fn allocate(&mut self) -> (ContextItem, usize);
	fn dispose(&mut self);
	fn fetch(&self, reference: usize) -> &ContextItem;
	fn fetch_mut(&mut self, reference: usize) -> &mut ContextItem;
}

impl Response {
	fn append(&mut self, actions: Vec<&dyn Action>);
	fn step(&mut self);
}

trait Labelled {
	fn get_label(&self) -> &str;
}