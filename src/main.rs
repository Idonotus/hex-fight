use rand::{Rng, RngCore};

use crate::{
    cards::{
        CardPicker,
        RLEDeck,
        SimpleCard,
        Stacks,
        Card,
        RandomDraw
    },
    colors::Color
};

mod cards;
mod colors;

struct Player {
    hand: Vec<Card>
}

struct Game {
    deck: Box<dyn RandomDraw>,
    card_set: CardPicker,

    players: Vec<Player>,
    current_turn: usize,
    direction: usize,

    top_card: Option<Card>,
    rng: Box<dyn RngCore>
}

impl Game {
    fn new(player_count: usize, rng: Box<dyn RngCore>) -> Self {
        let mut players = Vec::new();

        for _ in 0..player_count {
            players.push(
                Player { hand: Vec::new() }
            );
        }

        Self {
            deck: Box::new(RLEDeck::new(0xA000000)),
            card_set: card_generator,
            players,
            current_turn: 0,
            direction: 1,
            top_card: None,
            rng
        }
    }

    fn deal(&mut self, per_player: u64) {
        for _ in 0..per_player {
            for p in &mut self.players {
                let d= self.deck.as_mut();
                let card = d.draw_card_random(self.rng.as_mut()).unwrap();
                p.hand.push((self.card_set)(card));
            }
        }
    }
}

fn card_generator(card: u64) -> Box<dyn Stacks> {
    let (card, r) = (card / 256, (card % 256).try_into().unwrap());
    let (card, g) = (card / 256, (card % 256).try_into().unwrap());
    let (value, b) = ((card / 256).try_into().unwrap(), (card % 256).try_into().unwrap());
    
    Box::new(
        SimpleCard::new(
            Color {
                r,
                g,
                b
            },
            value
        )
    )
}

fn main() {
    println!("Hello, world!");
}
