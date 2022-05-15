use std::io;
use std::io::Write;

use super::common::{AnswerData, FromServerMessage, FromClientMessage, BYTECOUNT};
use crate::terminal::VisualDeck;

use message_io::network::{NetEvent, Transport, RemoteAddr};
use message_io::node::{self, NodeEvent};

use std::time::{Duration};
use termion::screen::AlternateScreen;
use crate::command::get_command;
use crate::common::{Card, CARDCOUNT, CardType, HandCardData};

enum Signal {
    Greet, // This is a self event called every second.
           // Other signals here,
}

pub fn run(transport: Transport, remote_addr: RemoteAddr) {
    let mut deck = VisualDeck::new();
    let default_hand: HandCardData = [ Card{ _type : CardType::Joker, value : 0} ; CARDCOUNT];
    let mut actual_hand = default_hand;
    let stdin = io::stdin();

    let mut answer_data = | hand : & HandCardData |{
        let mut screen = AlternateScreen::from(io::stdout());
        //write!(screen, "Writing to alternat(iv)e screen!").unwrap();
        screen.flush().unwrap();


        //let mut stdin = termion::async_stdin().keys();

        let mut buffer: AnswerData = [' '; BYTECOUNT];

        let mut opt_answer : Option<String> =  get_command(& mut deck, hand);

        for (i,ch) in opt_answer.unwrap().chars().enumerate() {
            if i < BYTECOUNT {
                buffer[i] = ch;
            } else {
                break;
            }
        }
        let message = FromClientMessage::TurnAnswer(buffer);
        bincode::serialize(&message).unwrap()

    };



    let (handler, listener) = node::split();

    let (server_id, local_addr) =
        handler.network().connect(transport, remote_addr.clone()).unwrap();

    listener.for_each(move |event| match event {
        NodeEvent::Network(net_event) => match net_event {
            NetEvent::Connected(_, established) => {
                if established {
                    println!("Connected to server at {} by {}", server_id.addr(), transport);
                    println!("Client identified by local port: {}", local_addr.port());
                    handler.signals().send(Signal::Greet);
                }
                else {
                    println!("Can not connect to server at {} by {}", remote_addr, transport);
                    handler.stop();
                }
            }
            NetEvent::Accepted(_, _) => unreachable!(), // Only generated when a listener accepts
            NetEvent::Message(endpoint , input_data) => {
                let message: FromServerMessage = bincode::deserialize(&input_data).unwrap();
                match message {
                    FromServerMessage::Pong(_) => {
                        let message = FromClientMessage::NewTurn;
                        let output_data = bincode::serialize(&message).unwrap();
                        handler.network().send(endpoint, &output_data);
                    },
                    FromServerMessage::UnknownPong => println!("Pong from server"),
                    
                    FromServerMessage::TurnBegin(hand) => {
                        actual_hand = hand;
                        handler.network().send(endpoint, & mut answer_data(&hand) );

                    },

                    FromServerMessage::TurnContinue => {
                        let mut screen = AlternateScreen::from(io::stdout());
                        write!(screen, "turn continue").unwrap();
                        handler.network().send(endpoint, & mut answer_data(&actual_hand) );
                    }

                    FromServerMessage::TurnEnd(_) =>
                        {
                            let mut screen = AlternateScreen::from(io::stdout());
                            write!(screen, "turn end!").unwrap();
                            let message = FromClientMessage::NewTurn;
                            let output_data = bincode::serialize(&message).unwrap();
                            handler.network().send(endpoint, &output_data);
                        },
                }
            }
            NetEvent::Disconnected(_) => {
                println!("Server is disconnected");
                handler.stop();
            }
        },
        NodeEvent::Signal(signal) => match signal {
            Signal::Greet => {
                let message = FromClientMessage::Ping;
                let output_data = bincode::serialize(&message).unwrap();
                handler.network().send(server_id, &output_data);
                handler.signals().send_with_timer(Signal::Greet, Duration::from_secs(1));
            }
        },
    });
}
