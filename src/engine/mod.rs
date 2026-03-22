use rand::RngCore;

mod actions;
pub mod cards;
pub mod colors;
mod scheduler;

use cards::{AssignedBand, BandSet, RLEDeck, RandomDraw};
use colors::{ColorComparison, build_dist_tolerance_eq, taxicab};
use scheduler::Scheduler;

use crate::engine::cards::{BaseBand, Stacks};

pub(crate) trait Predicate<T> {
    fn get_predicate(&self) -> T;
}

pub struct Player<Card> {
    pub hand: Vec<Card>,
}

pub struct Game<'a, Band, Card>
where
    Card: Stacks,
    Band: AssignedBand<'a, Card>,
{
    deck: Box<dyn RandomDraw>,
    rng: Box<dyn RngCore>,
    card_set: BandSet<'a, Band, Card>,

    players: Vec<Player<Card>>,
    order: Scheduler<'a>,

    top_card: Option<Card>,
    comparison: ColorComparison,
}

impl<'a, Band, Card> Game<'a, Band, Card>
where
    Card: Stacks,
    Band: AssignedBand<'a, Card>,
{
    pub fn new(
        player_count: usize,
        rng: Box<dyn RngCore>,
        card_set: BandSet<'a, Band, Card>,
    ) -> Self {
        let mut players = Vec::new();

        for _ in 0..player_count {
            players.push(Player { hand: Vec::new() });
        }

        Self {
            deck: Box::new(RLEDeck::new(card_set.get_band_size())),
            card_set,
            players,
            order: Scheduler::new(player_count),
            top_card: None,
            rng,
            comparison: build_dist_tolerance_eq(taxicab, 64),
        }
    }

    pub fn deal(&mut self, per_player: u64) {
        for _ in 0..per_player {
            for p in &mut self.players {
                let d = self.deck.as_mut();
                let card = d.draw_card_random(self.rng.as_mut()).unwrap();
                p.hand.push(self.card_set.generate_card(card));
            }
        }
    }

    pub fn get_player(&self, p: usize) -> &Player<Card> {
        return &self.players[p];
    }

    pub fn draw(&mut self, player: usize) -> bool {
        match self.draw_card() {
            None => {
                return false;
            }
            Some(card) => {
                self.players[player].hand.push(card);
                return true;
            }
        }
    }

    pub fn play_card(&mut self, player: usize, hand_number: usize) {
        self.top_card = Some(self.players[player].hand.remove(hand_number));
    }

    pub fn draw_card(&mut self) -> Option<Card> {
        match self.deck.draw_card_random(self.rng.as_mut()) {
            Some(identity) => Some(self.card_set.generate_card(identity)),
            None => None,
        }
    }

    pub fn get_valid_stacks_for_player(
        &self,
        player: usize,
        compare: &dyn Fn(&Card, &Card) -> bool,
    ) -> Vec<usize> {
        let cmp = self.top_card.as_ref().unwrap();
        return Vec::from_iter(self.players[player].hand.iter().enumerate().filter_map(
            |(idx, card)| {
                if compare(cmp, card) {
                    return Some(idx);
                }
                return None;
            },
        ));
    }

    pub fn get_current_player(&self) -> &Player<Card> {
        self.get_player(self.order.get_turn())
    }
}

// pub fn main() {
//     let mut game = Game::new(2, Box::new(rand::rng()));

//     game.deal(2026);
//     game.top_card = game.draw_card();
//     loop {
//         let mut phases: ActionQueue = game.order.pop_current_turn();
//         phases.run(TurnPhase::Start);

//         println!("Player {}'s turn: they have {} cards", game.order.get_turn() + 1, game.get_current_player().hand.len());
//         let t = game.top_card.as_ref().unwrap().as_ref();
//         println!("The top card is:\n{:?} {:?}\n\n", t.get_color(), t.get_value());

//         let opt = game.get_valid_stacks_for_player(game.order.get_turn(), &|base: &Card<'_>, head: &Card<'_>| {
//             does_stack(base, head, &game.comparison)
//         });

//         let p = game.get_current_player();
//         for (i, idxcard) in opt.iter().enumerate() {
//             let card = &p.hand[*idxcard];
//             println!("{}: {:?} {:?}", i+1, card.get_color(), card.get_value());
//         }
//         println!("{}: Draw a card", opt.len() + 1);
//         let mut imput = String::new();
//         io::stdin()
//         .read_line(&mut imput)
//         .expect("Failed to read line");

//         let index_input: usize = match imput.trim().parse::<usize>() {
//             Ok(n) => n - 1,
//             Err(_) => continue,
//         };

//         if index_input == opt.len() {
//             if !game.draw(game.order.get_turn()) {
//                 break;
//             }
//             phases.run(TurnPhase::Play);
//         } else if index_input > opt.len() {
//             println!("Enter a valid action!!!");
//             continue;
//         } else {
//             game.play_card(game.order.get_turn(), opt[index_input]);
//             phases.run(TurnPhase::Play);
//         }

//         if game.get_player(game.order.get_turn()).hand.len() == 0 {
//             break;
//         }

//         game.order.tick();
//         phases.run(TurnPhase::End);
//     }
// }
