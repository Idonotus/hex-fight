use std::io;
use rand::RngCore;

use crate::{
    cards::{
        CardPicker,
        RLEDeck,
        SimpleCard,
        Card,
        RandomDraw,
    },
    colors::{
        Color,
        ColorComparison,
        build_dist_tolerance_eq,
        taxicab
    }
};

mod cards;
mod colors;

struct Player {
    hand: Vec<Card>
}

struct Game {
    deck: Box<dyn RandomDraw>,
    rng: Box<dyn RngCore>,
    card_set: CardPicker,

    players: Vec<Player>,
    order: TurnOrder,

    top_card: Option<Card>,
    comparason: ColorComparison,
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
            order: TurnOrder::new(player_count),
            top_card: None,
            rng,
            comparason: build_dist_tolerance_eq(taxicab, 64),
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

    fn get_player(&self, p: usize) -> &Player {
        return &self.players[p]
    }

    fn draw(&mut self, player: usize) -> bool {
        match self.draw_card() {
            None => {return false;}
            Some(card) => {
                self.players[player].hand.push(card);
                return true;
            }
        }
    }

    fn play_card(&mut self, player: usize, hand_number: usize) -> bool {
        match &self.top_card {
            None => {
                self.top_card = Some(self.players[player].hand.remove(hand_number));
                return true;
            }
            Some(top_card) => {
                let card = &self.players[player].hand[hand_number];
                if !card.does_stack(top_card.as_ref(), &self.comparason) {
                    return false;
                }
                self.top_card = Some(self.players[player].hand.remove(hand_number));
                return true;
            }
        }
    }
    
    fn draw_card(&mut self) -> Option<Card> {
        match self.deck.draw_card_random(self.rng.as_mut()) {
            Some(identity) => Some((self.card_set)(identity)),
            None => None
        }
    }
}

fn card_generator(card: u64) -> Card {
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

#[derive(Debug)]
struct TurnOrder {
    order: Vec<usize>,
    queue: Vec<()>,
    tracker: usize,
    direction: i8,
}

impl TurnOrder {
    fn new(player_count: usize) -> Self {
        Self { order: Vec::from_iter(0..player_count), queue: Vec::new(), tracker: 0, direction: 1 }
    }

    fn reverse(&mut self) {
        self.direction = -self.direction
    }

    fn tick(&mut self) {
        let a: i64 = self.tracker.try_into().unwrap();
        let b : i64 = self.direction.into();
        let c: i64 = self.order.len().try_into().unwrap();

        self.tracker = ((a + b) % c).try_into().unwrap();
    }

    fn skip(&mut self) {
        todo!()
    }

    fn get_turn(&self) -> usize {
        self.order[self.tracker]
    }
}

fn main() {
    let mut game = Game::new(2, Box::new(rand::rng()));

    game.deal(7);
    game.top_card = game.draw_card();
    loop {
        println!("Player {}'s turn", game.order.get_turn() + 1);
        let t = game.top_card.as_ref().unwrap().as_ref();
        println!("The top card is:\n{:?} {:?}\n\n", t.get_color(), t.get_value());

        let p = game.get_player(game.order.get_turn());
        for (i, card) in p.hand.iter().enumerate() {
            println!("{}: {:?} {:?}", i+1, card.get_color(), card.get_value());
        }
        println!("{}: Draw a card", p.hand.len() + 1);
        let mut imput = String::new();
        io::stdin()
        .read_line(&mut imput)
        .expect("Failed to read line");
        
        let index_input: usize = match imput.trim().parse::<usize>() {
            Ok(n) => n - 1,
            Err(_) => continue,
        };

        if index_input == p.hand.len() {
            if !game.draw(game.order.get_turn()) {
                break;
            }
            game.order.tick();
            continue;
        }
        if index_input > p.hand.len() {
            println!("Enter a valid action!!!");
            continue;
        }

        if !game.play_card(game.order.get_turn(), index_input) {
            continue;
        }
        if game.get_player(game.order.get_turn()).hand.len() == 0 {
            break;
        }
        game.order.tick();
    }
}
