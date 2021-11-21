use serde::{Serialize, Deserialize};
//use common_game::{Card};

#[derive(Serialize, Deserialize)]
pub enum FromClientMessage {
    Ping,
    Answer,
}

#[derive(Serialize, Deserialize)]
pub enum FromServerMessage {
    Pong(usize),            // Used for connection oriented protocols
    UnknownPong,            // Used for non-connection oriented protocols
    TurnYouWin,             // Used for bring a good notice
    TurnOtherWin,       // Used for bring a bad notice (and say how do a good notice)
    TurnTied,               // Used for bring a bad notice for all
    TurnBegin/*([Card; 4])*/,   // Used for bring the cards

    
}
