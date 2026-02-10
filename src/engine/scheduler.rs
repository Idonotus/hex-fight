use std::{
	ops
};
use enum_dict::{
	RequiredDict,
	DictKey
};

struct Task<'a> {
	phase: TurnPhase,
	callback: Box<dyn Fn() -> () + 'a>,
}

trait TimerTask<'a> {
	fn is_activating(&self) -> bool;
	fn can_prune(&self) -> bool;
	fn get_task(&self) -> &Task<'a>;
	fn own_task(self) -> Task<'a>;
}

struct PlayTask<'a> {
	tracker: Tracker,

	task: Task<'a>
}

impl<'a> TimerTask<'a> for PlayTask<'a> {
	fn is_activating(&self) -> bool {
		self.tracker.is_overdue()
	}

	fn can_prune(&self) -> bool {
		self.tracker.is_overdue()
	}

	fn get_task(&self) -> &Task<'a> {
		&self.task
	}

	fn own_task(self) -> Task<'a> {
		self.task
	}
}

struct RelativeTask<'a> {
	tracker: Tracker,
	pinpoint: bool,

	task: Task<'a>
}

impl<'a> TimerTask<'a> for RelativeTask<'a> {
	fn is_activating(&self) -> bool {
		if self.pinpoint {
			self.tracker.is_done()
		} else {
			self.tracker.is_overdue()
		}
	}

	fn can_prune(&self) -> bool {
		if self.pinpoint {
			return self.tracker.is_overdue();
		}
		return false;
	}

	fn get_task(&self) -> &Task<'a> {
		&self.task
	}

	fn own_task(self) -> Task<'a> {
		self.task
	}
}

#[derive(DictKey, Clone, Copy)]
pub enum TurnPhase {
	Start,
	Play,
	End,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Tracker {
	turns: isize
}

impl Tracker {
	pub fn new(turns: isize) -> Self {
		Self { turns }
	}

	fn is_done(&self) -> bool {
		self.turns == 0
	}

	fn is_overdue(&self) -> bool {
		self.turns > 0 || self.is_done()
	}
}

struct TurnCounter {
	plays: usize,
	turns: usize
}

impl TurnCounter {
	fn new() -> Self {
		Self { plays: 0, turns: 0 }
	}
}

pub struct Scheduler<'a> {
    order: Vec<usize>,
	tracker: TurnCounter,
	cur_turn: usize,
    direction: isize,

    queue: Vec<RelativeTask<'a>>,
	play_queue: Vec<PlayTask<'a>>,
	playerqueue: Vec<Vec<Task<'a>>>,
}

impl<'a> Scheduler<'a> {
    pub fn new(player_count: usize) -> Self {
        let mut playerqueue = Vec::new();
		for _ in 0..player_count {
			playerqueue.push(Vec::new());
		}
		Self {
			order: Vec::from_iter(0..player_count),
			queue: Vec::new(),
			play_queue: Vec::new(),
			playerqueue,
			tracker: TurnCounter::new(),
			direction: 1,
			cur_turn: 0
		}
    }

    pub fn reverse(&mut self) {
        self.direction = -self.direction
    }

	pub fn go(&mut self, amount: usize) {
		let samount: isize = amount.try_into().unwrap();
;
		let i: isize = self.cur_turn.try_into().unwrap();
        let b : isize = self.direction * samount;
        let c: isize = self.order.len().try_into().unwrap();

        self.cur_turn = ((i + b) % c).try_into().unwrap();
		
		self.tracker.turns += amount;
		for t in &mut self.queue {
			t.tracker.turns -= samount;
		}
	}

    pub fn tick(&mut self) {
        self.go(1);

		self.tracker.turns += 1;
		for t in &mut self.play_queue {
			t.tracker.turns -= 1;
		}
    }

    pub fn get_turn(&self) -> usize {
        self.order[self.cur_turn]
    }

	pub fn push_player_task(&mut self, player: usize, task: Task<'a>) {
		self.playerqueue[player].push(task);
	}

	pub fn schedule_task(&mut self, task: RelativeTask<'a>) {
		self.queue.push(task);
	}

	pub fn pop_current_turn(&mut self) -> ActionQueue<'a> {
		let mut phases: ActionQueue<'_> = ActionQueue(RequiredDict::from_fn(|_| Vec::new()));

		let player = self.get_turn();
		for t in self.playerqueue[player].drain(..) {
			phases.0[t.phase].push(t.callback);
		}

		fetch_and_prune(&mut self.play_queue, &mut phases);
		fetch_and_prune(&mut self.queue, &mut phases);

		return phases;
	}
}

fn fetch_and_prune<'a>(tasks: &mut Vec<impl TimerTask<'a>>, queue: &mut ActionQueue<'a>) {
	for i in (0..tasks.len()).rev() {
		if !tasks[i].can_prune() {continue;}
		let task_timer = tasks.remove(i);
		if !task_timer.is_activating() {continue;}
		let task = task_timer.own_task();
		queue.0[task.phase].push(task.callback);
	}
}

pub struct ActionQueue<'a>(RequiredDict<TurnPhase, Vec<Box<dyn FnOnce() -> () + 'a>>>);

impl ActionQueue<'_> {
	pub fn run(&mut self, phase: TurnPhase) {
		for t in self.0[phase].drain(..) {
			t()
		}
	}
}