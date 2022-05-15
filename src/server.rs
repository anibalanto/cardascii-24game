use std::any::Any;
use super::common::{FromServerMessage, FromClientMessage};

use message_io::network::{NetEvent, Transport, Endpoint};
use message_io::node::{self};

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::iter::FromIterator;
use std::net::{SocketAddr};
use nom::ToUsize;
use rcalc::{ASTNode, Interpreter, Lexer, Parser, Token};
use crate::answer_analizer;
use crate::common::{Card, CARDCOUNT, CardType, HandCardData, TurnEndType};
use crate::core_cards::{Game24, TurnResult};

struct ClientInfo {
    id: usize,
}

pub fn run(transport: Transport, addr: SocketAddr) {
    let (handler, listener) = node::split::<()>();

    let mut clients= HashMap::<Endpoint, ClientInfo>::new();
    let mut id = 0;

    let mut game = Game24::new();

    let mut cards: HandCardData =
        [ Card{ _type : CardType::Joker, value : 0} ; CARDCOUNT];

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
                FromClientMessage::TurnAnswer(user_id, entry) => {
                    let answer = String::from_iter(entry);
                    println!("user:say >> {}", answer);

                    let mut message = FromServerMessage::TurnContinue;



                    // FIXME llevar esto tambien a game::make_answer
                    let mut program = Interpreter::from(answer.as_str());

                    if let Ok( result  ) = program.interpret() { //answer_analizer::analize(&answer) {
                        println!("@ {}", result);
                        if result == 24.0 {
                            message = match game.make_answer(user_id, answer) {
                                Ok(()) =>       FromServerMessage::TurnEnd(TurnEndType::YouWin),
                                Err(msgErr) =>  FromServerMessage::SendMsg(msgErr)
                            }
                        }
                    }
                    println!("{message}");
                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                }

            }
        },
        NetEvent::Disconnected(endpoint) => {}
    });
}
