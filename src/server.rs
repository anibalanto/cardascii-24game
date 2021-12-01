use super::common::{FromServerMessage, FromClientMessage};

use message_io::network::{NetEvent, Transport, Endpoint};
use message_io::node::{self};

use std::collections::{HashMap};
use std::iter::FromIterator;
use std::net::{SocketAddr};
use crate::answer_analizer;
use crate::common::{Card, CARDCOUNT, CardType, HandCardData, TurnEndType};
use crate::core_cards::{Game24, TurnResult};

struct ClientInfo {
    id: usize,
}

pub fn run(transport: Transport, addr: SocketAddr) {
    let (handler, listener) = node::split::<()>();

    let mut clients: HashMap<Endpoint, ClientInfo> = HashMap::new();
    let mut id = 0;

    let mut game = Game24::new();

    match handler.network().listen(transport, addr) {
        Ok((_id, real_addr)) => println!("Server running at {} by {}", real_addr, transport),
        Err(_) => return println!("Can not listening at {} by {}", addr, transport),
    }

    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => (), // Only generated at connect() calls.
        NetEvent::Accepted(endpoint, _listener_id) => {
            // Only connection oriented protocols will generate this event

            clients.insert(endpoint, ClientInfo { id }); id += 1;

            println!("Client ({}) connected (total clients: {})", endpoint.addr(), clients.len());
        }
        NetEvent::Message(endpoint, input_data) => {
            let message: FromClientMessage = bincode::deserialize(&input_data).unwrap();
            match message {
                FromClientMessage::Ping => {
                    let message = match clients.get_mut(&endpoint) {
                        Some(client) => {
                            // For connection oriented protocols
                            println!("Ping from {}, {} times", endpoint.addr(), client.id);
                            FromServerMessage::Pong(client.id)
                        }
                        None => {
                            // For non-connection oriented protocols
                            println!("Ping from {}", endpoint.addr());
                            FromServerMessage::UnknownPong
                        }
                    };
                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                },
                FromClientMessage::NewTurn => {
                    if game.give_cards() {

                        let mut cards: HandCardData =
                            [ Card{ _type : CardType::Joker, value : 0} ; CARDCOUNT];

                        for i in 0..CARDCOUNT {
                            if let Some(card) = game.get_gived_card(i) {
                                cards[i] = card.clone();
                                println!("{:?}", card);
                            }
                        }
                        let message = FromServerMessage::TurnBegin( cards );
                        let output_data = bincode::serialize(&message).unwrap();
                        handler.network().send(endpoint, &output_data);
                    }
                }
                FromClientMessage::TurnAnswer(entry) => {
                    let answer = String::from_iter(entry);
                    println!("user:say >> {}", answer);

                    let mut message = FromServerMessage::TurnContinue;

                    if let Ok( ( _ , result ) ) = answer_analizer::analize(&answer) {
                        println!("@ {}", result);
                        if result == 24 {
                            game.end_turn(TurnResult::Winner(0));
                            message = FromServerMessage::TurnEnd(TurnEndType::YouWin)
                        }
                    }

                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                }

            }
        }
        NetEvent::Disconnected(endpoint) => {
            // Only connection oriented protocols will generate this event
            clients.remove(&endpoint).unwrap();
            println!(
                "Client ({}) disconnected (total clients: {})",
                endpoint.addr(),
                clients.len()
            );
        }
    });
}
