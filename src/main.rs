extern crate termion;

use termion::{event::{Key, Event, MouseEvent}, raw::RawTerminal};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;
use std::{io::{Write, stdout, stdin, Stdout, Stdin}, net::ToSocketAddrs};

use rand::thread_rng;
use rand::seq::SliceRandom;

enum Palo { //card type 
    Oro,    //gold
    Basto,  //club
    Espada, //sword
    Copa,   //cup
    Comodin //joker
}

const CARD_ID_JOCKER_1: u8 = 0;
const CARD_ID_JOCKER_2: u8 = 1;

struct Card {
    palo: Palo,
    value: u8,
    visual: Vec<&'static str>
}

fn draw_card(card_visual: &Vec<&'static str>, stdout: &mut MouseTerminal<RawTerminal<Stdout>>, (x, y): &(u16, u16)) {
    let mut row = *y;
    for str in card_visual {
        write!(stdout, "{}{}", termion::cursor::Goto(*x, row), str).unwrap();
        row +=1;
    }
}


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
    cards: Vec<Card>,
    back: Vec<&'static str>
}

impl Deck {
    fn new() -> Self {
        let mut me = Deck {
            cards : Vec::<Card>::new(),
            back : card_str_back()
        };
        load_cards_str_front(& mut me);
        me
    }
    
    fn agregar(&mut self, palo: Palo, value: u8, visual: Vec<&'static str>) {
        self.cards.push(Card{
            palo, value, visual
        });
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

}

struct CardStack {
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

struct Game24 <'a>{
    player:         u8,
    deck:           &'a Deck,
    hidden_cards:   CardStack,
    visible_cards:  CardStack,
    player1_cards:  CardStack,
    player2_cards:  CardStack,
    accumulate_cards:  CardStack,
    operation:      String,
    turn_num:       u8
}
#[derive(PartialEq)]
enum GameResult {Win, Lose, Tie, Gaming, Abandoned}

impl<'a> Game24<'a> {
    fn new(player: u8, deck: &'a Deck) -> Self {
        let mut hidden_cards = CardStack::new(false);
        hidden_cards.add_cards(deck);
        hidden_cards.shuffle();

        Game24 {
            player,
            deck,
            hidden_cards,
            visible_cards:  CardStack::new(true),
            player1_cards:  CardStack::new(false),
            player2_cards:  CardStack::new(false),
            accumulate_cards:  CardStack::new(false),
            operation:      "24".to_string(),
            turn_num: 0
        }
    }

    fn reset(&mut self) {
        self.hidden_cards.add_all_from( &mut self.visible_cards );
        self.hidden_cards.add_all_from( &mut self.player1_cards );
        self.hidden_cards.add_all_from( &mut self.player2_cards );

        self.hidden_cards.shuffle();
    }

    fn play(&mut self) -> GameResult{
        let mut stdin = stdin();
        let mut stdout = MouseTerminal::from(stdout().into_raw_mode().unwrap());
    
        stdout.flush().unwrap();

        let mut result : GameResult;
        loop {
            result = self.turn(&mut stdin, &mut stdout);
            if result != GameResult::Gaming {
                break;
            }
        }
        result
    }

    fn turn(&mut self, stdin: &mut Stdin, stdout: &mut MouseTerminal<RawTerminal<Stdout>>) -> GameResult{
        write!(stdout, "{}{}turn: {} (push 'r' for next turn)", termion::clear::All, termion::cursor::Goto(1, 1), self.turn_num).unwrap();
        let mut result =  GameResult::Gaming;
        if  ! self.hidden_cards.is_empty()
            &&self.visible_cards.add_n_from(&mut self.hidden_cards, 4) {
            let mut positions = Vec::<(u16, u16)>::new();
            positions.push((2,2));
            positions.push((20,2));
            positions.push((2,12));
            positions.push((20,12));
            let mut pos_iter = positions.iter();
            for card_id in self.visible_cards.card_ids.iter() {
                if let Some(pos) = pos_iter.next() {
                    if let Some(card) = self.deck.get_card(card_id) {
                        draw_card(& card.visual, stdout, pos);
                    }
                }
            }
            stdout.flush().unwrap();

            match self.play_turn(stdin) {
                GameResult::Win =>
                    self.player2_cards.add_all_from(&mut self.visible_cards),
                
                GameResult::Lose => 
                    self.player1_cards.add_all_from(&mut self.visible_cards),
                
                GameResult::Tie =>
                    self.accumulate_cards.add_all_from(&mut self.visible_cards),
                
                GameResult::Abandoned =>
                    result = GameResult::Abandoned,
                
                GameResult::Gaming => ()
            }
            self.turn_num += 1;
        } else {
            result = GameResult::Win;
        }
        result
    }

    fn play_turn(&mut self, stdin: &mut Stdin) -> GameResult{
        let mut turn = GameResult::Gaming;
        while turn == GameResult::Gaming {
            for c in stdin.events() {
                let evt = c.unwrap();
                match evt {
                    Event::Key(Key::Char('r')) => {
                        match self.resolve_operation() {
                            Some(op) =>
                                turn = if op == 24
                                    { GameResult::Win } else
                                    { GameResult::Lose },
                            None => turn = GameResult::Tie
                        };
                        break;
                    },
                    _ => {}
                }
            }
        }
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
    let deck = Deck::new();
    let mut game = Game24::new(0, &deck);
    game.play();
}

fn card_str_back() -> Vec<&'static str>{
    make_str_card!(
        r#"┌────────────┐"#,
        r#"│╳╳╳╳╳╳╳╳╳╳╳╳│"#,
        r#"│╳╳╳╳╳╳╳╳╳╳╳╳│"#,
        r#"│╳╳╳╳╳╳╳╳╳╳╳╳│"#,
        r#"│╳CARDASCII!╳│"#,
        r#"│╳╳╳╳╳╳╳╳╳╳╳╳│"#,
        r#"│╳╳╳╳╳╳╳╳╳╳╳╳│"#,
        r#"│╳╳╳╳╳╳╳╳╳╳╳╳│"#,
        r#"└────────────┘"#)
}

fn load_cards_str_front(mazo : & mut Deck) {
    mazo.agregar(Palo::Comodin, 0, make_str_card!(
        r#"┌────────────┐"#,
        r#"│J    ◔   ⊙  │"#,
        r#"│O  ๏ |\  |\ │"#,
        r#"│K  |\/ |/ | │"#,
        r#"│E  ʕ  ͡o  ͡o| │"#,
        r#"│R  °༽   ͜ʖ༼  │"#,
        r#"│     ༽  ༼   │"#,
        r#"│            │"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Comodin, 0, make_str_card!(
        r#"┌────────────┐"#,
        r#"│J    ◔   ⊙  │"#,
        r#"│O  ๏ |\  |\ │"#,
        r#"│K  |\/ |/ | │"#,
        r#"│E  ʕ  ͡o  ͡o| │"#,
        r#"│R  °༽   ͜ʖ༼  │"#,
        r#"│     ༽  ༼   │"#,
        r#"│            │"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Espada, 12, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│12  /^^^┼^\ │"#,
        r#"│|\ (  ° ͜ʖ° )│"#,
        r#"│ \\ \     / │"#,
        r#"│ _\\_---⚙-\ │"#,
        r#"│   ฿   .๏. \│"#,
        r#"│  /    .๏.  │"#,
        r#"│ /     .๏.12│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 11, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│11    ┌──@─┐│"#,
        r#"│|\    (° ͜ʖ°)│"#,
        r#"│ \\   /    \│"#,
        r#"│ _\\_Λ  Λ   │"#,
        r#"│   ฿(⚙  ⚙)\~│"#,
        r#"│     )  (  \│"#,
        r#"│     (..) 11│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 10, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│10   ┌───@┐ │"#,
        r#"│     │____│ │"#,
        r#"│  |\ (° ͜ʖ°) │"#,
        r#"│   \\/    \ │"#,
        r#"│   _\\_   / │"#,
        r#"│     ฿\  /฿ │"#,
        r#"│       || 10│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 9, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│9           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           9│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 8, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│8           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           8│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar( Palo::Espada, 7, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│7           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           7│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 6, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│6           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           6│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 5, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│5           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           5│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 4, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│4           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           4│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 3, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│3           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           3│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 2, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│2           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           2│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Espada, 1, make_str_card!(
        r#"┌──  ────  ──┐"#,
        r#"│1           │"#,
        r#"│            │"#,
        r#"│   |\       │"#,
        r#"│    \\      │"#,
        r#"│    _\\_    │"#,
        r#"│      \     │"#,
        r#"│           1│"#,
        r#"└──  ────  ──┘"#)
    );
    mazo.agregar(Palo::Basto, 12, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│12  /^^^┼^\ │"#,
        r#"│.-.(  ° ͜ʖ° )│"#,
        r#"│(  )\     / │"#,
        r#"│ ( )/---⚙-\ │"#,
        r#"│  ()   .๏. \│"#,
        r#"│  /    .๏.  │"#,
        r#"│ /     .๏.12│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 11, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│11    ┌──@─┐│"#,
        r#"│.-.   (° ͜ʖ°)│"#,
        r#"│(  )  /    \│"#,
        r#"│ ( ) Λ  Λ   │"#,
        r#"│  ()(⚙  ⚙)\~│"#,
        r#"│     )  (  \│"#,
        r#"│     (..) 11│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 10, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│10   ┌───@┐ │"#,
        r#"│.-.  │____│ │"#,
        r#"│(  ) (° ͜ʖ°) │"#,
        r#"│ ( ) /    \ │"#,
        r#"│  ฿)/\    / │"#,
        r#"│      \  /฿ │"#,
        r#"│       || 10│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 9, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│9           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           9│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 8, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│8           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           8│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 7, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│7           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           7│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 6, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│6           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           6│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 5, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│5           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           5│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 4, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│4           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           4│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 3, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│3           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           3│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 2, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│2           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           2│"#,
        r#"└─  ──  ──  ─┘"#)
    );
    mazo.agregar(Palo::Basto, 1, make_str_card!(
        r#"┌─  ──  ──  ─┐"#,
        r#"│1           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│    (  )    │"#,
        r#"│     ( )    │"#,
        r#"│      ()    │"#,
        r#"│           1│"#,
        r#"└─  ──  ──  ─┘"#)
    );

    mazo.agregar(Palo::Oro, 12, make_str_card!(
        r#"┌────────────┐"#,
        r#"│12  /^^^┼^\ │"#,
        r#"│   (  ° ͜ʖ° )│"#,
        r#"│ .-.\     / │"#,
        r#"│( O )---⚙-\ │"#,
        r#"│ `฿`   .๏. \│"#,
        r#"│  /    .๏.  │"#,
        r#"│ /     .๏.12│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 11, make_str_card!(
        r#"┌────────────┐"#,
        r#"│11    ┌──@─┐│"#,
        r#"│ .-.  (° ͜ʖ°)│"#,
        r#"│( O ) /    \│"#,
        r#"│ `-฿ Λ  Λ   │"#,
        r#"│    (⚙  ⚙)\~│"#,
        r#"│     )  (  \│"#,
        r#"│     (..) 11│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 10, make_str_card!(
        r#"┌────────────┐"#,
        r#"│10   ┌───@┐ │"#,
        r#"│     │____│ │"#,
        r#"│ .-. (° ͜ʖ°) │"#,
        r#"│( O )/    \ │"#,
        r#"│ `฿` \    / │"#,
        r#"│      \  /฿ │"#,
        r#"│       || 10│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 9, make_str_card!(
        r#"┌────────────┐"#,
        r#"│9           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           9│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 8, make_str_card!(
        r#"┌────────────┐"#,
        r#"│8           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           8│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 7, make_str_card!(
        r#"┌────────────┐"#,
        r#"│7           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           7│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 6, make_str_card!(
        r#"┌────────────┐"#,
        r#"│6           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           6│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 5, make_str_card!(
        r#"┌────────────┐"#,
        r#"│5           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           5│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 4, make_str_card!(
        r#"┌────────────┐"#,
        r#"│4           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           4│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 3, make_str_card!(
        r#"┌────────────┐"#,
        r#"│3           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           3│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 2, make_str_card!(
        r#"┌────────────┐"#,
        r#"│2           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           2│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Oro, 1, make_str_card!(
        r#"┌────────────┐"#,
        r#"│1           │"#,
        r#"│            │"#,
        r#"│    .-.     │"#,
        r#"│   ( O )    │"#,
        r#"│    `-`     │"#,
        r#"│            │"#,
        r#"│           1│"#,
        r#"└────────────┘"#)
    );
    mazo.agregar(Palo::Copa, 12, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│12  /^^^┼^\ │"#,
        r#"│   (  ° ͜ʖ° )│"#,
        r#"│ ___\     / │"#,
        r#"│(___)---⚙-\ │"#,
        r#"│ ฿_/   .๏. \│"#,
        r#"│  /    .๏.  │"#,
        r#"│ /     .๏.12│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 11, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│11    ┌──@─┐│"#,
        r#"│ ___  (° ͜ʖ°)│"#,
        r#"│(___) /    \│"#,
        r#"│ \_฿ Λ  Λ   │"#,
        r#"│    (⚙  ⚙)\~│"#,
        r#"│     )  (  \│"#,
        r#"│     (..) 11│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 10, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│10   ┌───@┐ │"#,
        r#"│     │____│ │"#,
        r#"│ ___ (° ͜ʖ°) │"#,
        r#"│(___)/    \ │"#,
        r#"│ \_฿ \    / │"#,
        r#"│      \  /฿ │"#,
        r#"│       || 10│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 9, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│9           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           9│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 8, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│8           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           8│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 7, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│7           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           7│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 6, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│6           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           6│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 5, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│5           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           5│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 4, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│4           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           4│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 3, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│3           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           3│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 2, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│2           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           2│"#,
        r#"└────    ────┘"#)
    );
    mazo.agregar(Palo::Copa, 1, make_str_card!(
        r#"┌────    ────┐"#,
        r#"│1           │"#,
        r#"│            │"#,
        r#"│    ___     │"#,
        r#"│   (___)    │"#,
        r#"│    \_/     │"#,
        r#"│            │"#,
        r#"│           1│"#,
        r#"└────    ────┘"#)
    );
}
