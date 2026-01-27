use crate::cards::{IdDeck, CardPicker, Stacks};

mod cards;
mod colors;

struct Player {
    hand: Vec<Box<dyn Stacks>>
}

struct Game {
    deck: Box<dyn IdDeck>,
    card_set: CardPicker,

    players: Vec<Player>,
    current_turn: usize,
    direction: usize,

    top_card: Box<dyn Stacks>
}

fn main() {
    println!("Hello, world!");
}
