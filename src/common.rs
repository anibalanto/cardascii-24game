use serde::{Serialize, Deserialize};

extern crate serde;
extern crate bincode;


#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum CardType {
    Gold,
    Club,
    Sword,
    Cup,
    Joker
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Card {
    pub(crate) _type: CardType,
    pub(crate) value: u8
}

pub const BYTECOUNT: usize = 32;
pub type AnswerData = [char; BYTECOUNT];

pub(crate) const CARDCOUNT: usize = 4;
pub type HandCardData = [Card; CARDCOUNT];


#[derive(Serialize, Deserialize)]
pub enum FromClientMessage {
    Ping,
    NewTurn,
    TurnAnswer(usize, AnswerData),
}

#[derive(Serialize, Deserialize)]
pub enum TurnEndType {
    Tie,
    YouWin,
    OtherWin
}

#[derive(Serialize, Deserialize)]
pub enum FromServerMessage {

    Pong(usize),            // Used for connection oriented protocols
    UnknownPong,            // Used for non-connection oriented protocols
    TurnEnd(TurnEndType),             // Used for bring a good notice
    SendMsg(str),
    TurnContinue,               // Used for bring a bad notice for all
    TurnBegin(HandCardData),   // Used for bring the cards

    
}
