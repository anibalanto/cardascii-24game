extern crate termion;

use std::io::{stdin, Stdin, stdout, Write};
use rand::prelude::SliceRandom;
use rand::thread_rng;


use super::common::{Card, CardType};
use self::termion::input::{MouseTerminal, TermRead};
use self::termion::raw::{IntoRawMode, RawTerminal};

const CARD_ID_JOCKER_1: u8 = 0;
const CARD_ID_JOCKER_2: u8 = 1;


/*macro_rules! make_str_card {
    ( $x:expr $( , $more:expr )* ) => (
        format!("{}{}{}{}{}{}{}{}{}", $x, $( $more ),* )
    )
}*/

macro_rules! make_str_card {
    ( $( $x:expr ),* ) => {
        {
            let mut vec = Vec::<&'static str>::new();
            $(
                #[allow(unused_assignments)]
                {
                    vec.push($x);
                }
            )*
            vec
        }
    };
}

struct Deck{
    cards: Vec<Card>
}

impl Deck {
    fn new() -> Self {

        //static mut VISUAL_CARDS : HashMap<Card, Vec<&'static str>> = HashMap::<Card, Vec<&'static str>>::new();

        let mut me = Deck {
            cards : Vec::<Card>::new(),
        };

        load_cards(& mut me);

        me
    }

    fn add(&mut self, _type: CardType, value: u8) {
        self.cards.push( Card{ _type, value } );
    }

    fn as_ids(& self) -> Vec<u8> {
        (0 .. self.cards.len() as u8).collect()
    }

    fn as_ids_no_jokers(& self) -> Vec<u8> {
        (0 .. self.cards.len() as u8).collect()
    }

    fn get_card(& self, id: & u8) -> Option<&Card> {
        self.cards.get(*id as usize)
    }

    fn get_card_pos(& self, i: usize, card_ids : & Vec<u8>) -> Option<&Card> {
        match card_ids.get(i) {
            Some(id)    => self.get_card(id),
            None            => None
        }
    }

}

pub struct CardStack {
    is_face_up:bool,
    card_ids: Vec<u8>,
}

impl CardStack {
    fn new(is_face_up: bool) -> Self {
        CardStack {
            is_face_up,
            card_ids: Vec::<u8>::new()
        }
    }

    fn add_cards(&mut self, deck: &Deck) {
        self.card_ids = deck.as_ids_no_jokers();
    }

    fn add_all_from(&mut self, from: &mut CardStack) {
        self.card_ids.append(& mut from.card_ids);
        //from.card_ids.clear();
    }

    fn add_one_from(&mut self, from: &mut CardStack) -> bool {
        let mut result = false;
        if let Some(id) = from.card_ids.pop() {
            self.card_ids.push(id);
            result = true;
        }
        result
    }

    fn add_n_from(&mut self, from: &mut CardStack, n: u8) -> bool {
        let mut result = true;
        for _ in 0..n {
            if !self.add_one_from(from) {
                result = false;
                break
            }
        }
        result
    }

    fn shuffle(&mut self) {
        self.card_ids.shuffle(&mut thread_rng());
    }

    fn is_empty(&self) -> bool {
        self.card_ids.is_empty()
    }

}

pub struct Game24{
    //player:             u8,
    deck:               Deck,
    hidden_cards:       CardStack,
    visible_cards:  CardStack,
    players_cards:      Vec<CardStack>,
    accumulate_cards:   CardStack,
    operation:          String,
    turn_num:           u8
}
#[derive(PartialEq)]
pub enum TurnResult {
    Winner(usize),
    Tie,
    Gaming,
    Abandoned
}

impl Game24 {
    pub(crate) fn new(/*player: u8, deck: &'a Deck*/) -> Self {
        let mut hidden_cards = CardStack::new(false);
        let mut deck = Deck::new();
        hidden_cards.add_cards(&deck);
        hidden_cards.shuffle();
        let mut players_cards = Vec::<CardStack>::new();

        players_cards.push(CardStack::new(false));
        players_cards.push(CardStack::new(false));

        Game24 {
            deck,
            hidden_cards,
            visible_cards:  CardStack::new(true),
            players_cards,
            accumulate_cards:  CardStack::new(false),
            operation:      "24".to_string(),
            turn_num: 0
        }
    }

    fn reset(&mut self) {
        self.hidden_cards.add_all_from( &mut self.visible_cards );
        for player_cards in & mut self.players_cards.iter_mut() {
            self.hidden_cards.add_all_from( player_cards );
        }

        self.hidden_cards.shuffle();
    }

    fn play(&mut self) -> TurnResult {
        let mut stdin = stdin();
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());

        stdout.flush().unwrap();

        let mut result : TurnResult = TurnResult::Gaming;
      /*  loop {
            result = self.turn();
            if result != GameResult::Gaming {
                break;
            }
        }*/
        result
    }

    pub fn get_gived_card(&self, i: usize) -> Option<&Card>{
        self.deck.get_card_pos(i, &self.visible_cards.card_ids)
    }

    pub fn end_turn(&mut self, result: TurnResult) {
        //write!(stdout, "{}{}turn: {} (push 'r' for next turn)", termion::clear::All, termion::cursor::Goto(1, 1), self.turn_num).unwrap();
        match result {
            TurnResult::Winner(user) => {
                self.players_cards[user].add_all_from(&mut self.accumulate_cards);
                self.players_cards[user].add_all_from(&mut self.visible_cards);
            }
            TurnResult::Tie =>
                self.accumulate_cards.add_all_from(&mut self.visible_cards),
            _ => ()
        }
    }
    pub fn give_cards(&mut self) -> bool{
        !self.hidden_cards.is_empty() &&
            self.visible_cards.add_n_from(&mut self.hidden_cards, 4)
    }

    fn play_turn(&mut self, stdin: &mut Stdin) -> TurnResult {
        let mut turn = TurnResult::Gaming;
        /*while turn == GameResult::Gaming {
            for c in stdin.events() {
                let evt = c.unwrap();
                match evt {
                    Event::Key(Key()::Char('r')) => {
                        match self.resolve_operation() {
                            Some(op) =>
                                turn = if op == 24
                                { GameResult::Win } else
                                { GameResult::Lose },
                            None =>
                                turn = GameResult::Tie
                        };
                        break;
                    },
                    _ => {}
                }
            }
        }*/
        turn
    }

    fn resolve_operation(& self) -> Option<u16> {
        match self.operation.parse::<u16>() {
            Ok(op) => Some(op),
            Err(_) => None
        }
    }

}

fn main() {

    /*let mut game = Game24::new(0, &deck);
    game.play();*/
}

fn load_cards(deck: & mut Deck) {

    deck.add(CardType::Joker, 0);
    deck.add(CardType::Joker, 0);

    deck.add(CardType::Sword, 12);
    deck.add(CardType::Sword, 11);
    deck.add(CardType::Sword, 10);
    deck.add(CardType::Sword, 9);
    deck.add(CardType::Sword, 8);
    deck.add(CardType::Sword, 7);
    deck.add(CardType::Sword, 6);
    deck.add(CardType::Sword, 5);
    deck.add(CardType::Sword, 4);
    deck.add(CardType::Sword, 3);
    deck.add(CardType::Sword, 2);
    deck.add(CardType::Sword, 1);

    deck.add(CardType::Club, 12);
    deck.add(CardType::Club, 11);
    deck.add(CardType::Club, 10);
    deck.add(CardType::Club, 9);
    deck.add(CardType::Club, 8);
    deck.add(CardType::Club, 7);
    deck.add(CardType::Club, 6);
    deck.add(CardType::Club, 5);
    deck.add(CardType::Club, 4);
    deck.add(CardType::Club, 3);
    deck.add(CardType::Club, 2);
    deck.add(CardType::Club, 1);

    deck.add(CardType::Gold, 12);
    deck.add(CardType::Gold, 11);
    deck.add(CardType::Gold, 10);
    deck.add(CardType::Gold, 9);
    deck.add(CardType::Gold, 8);
    deck.add(CardType::Gold, 7);
    deck.add(CardType::Gold, 6);
    deck.add(CardType::Gold, 5);
    deck.add(CardType::Gold, 4);
    deck.add(CardType::Gold, 3);
    deck.add(CardType::Gold, 2);
    deck.add(CardType::Gold, 1);

    deck.add(CardType::Cup, 12);
    deck.add(CardType::Cup, 11);
    deck.add(CardType::Cup, 10);
    deck.add(CardType::Cup, 9);
    deck.add(CardType::Cup, 8);
    deck.add(CardType::Cup, 7);
    deck.add(CardType::Cup, 6);
    deck.add(CardType::Cup, 5);
    deck.add(CardType::Cup, 4);
    deck.add(CardType::Cup, 3);
    deck.add(CardType::Cup, 2);
    deck.add(CardType::Cup, 1);

}
