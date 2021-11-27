use super::common::{FromServerMessage, FromClientMessage};

use message_io::network::{NetEvent, Transport, Endpoint};
use message_io::node::{self};

use std::collections::{HashMap};
use std::iter::FromIterator;
use std::net::{SocketAddr};

struct ClientInfo {
    count: usize,
}

pub fn run(transport: Transport, addr: SocketAddr) {
    let (handler, listener) = node::split::<()>();

    let mut clients: HashMap<Endpoint, ClientInfo> = HashMap::new();

    match handler.network().listen(transport, addr) {
        Ok((_id, real_addr)) => println!("Server running at {} by {}", real_addr, transport),
        Err(_) => return println!("Can not listening at {} by {}", addr, transport),
    }

    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => (), // Only generated at connect() calls.
        NetEvent::Accepted(endpoint, _listener_id) => {
            // Only connection oriented protocols will generate this event
            clients.insert(endpoint, ClientInfo { count: 0 });
            println!("Client ({}) connected (total clients: {})", endpoint.addr(), clients.len());
        }
        NetEvent::Message(endpoint, input_data) => {
            let message: FromClientMessage = bincode::deserialize(&input_data).unwrap();
            match message {
                FromClientMessage::Ping => {
                    let message = match clients.get_mut(&endpoint) {
                        Some(client) => {
                            // For connection oriented protocols
                            client.count += 1;
                            println!("Ping from {}, {} times", endpoint.addr(), client.count);
                            FromServerMessage::Pong(client.count)
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
                FromClientMessage::Game => {
                    println!("Begin the game!");
                    let message = FromServerMessage::TurnBegin;
                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                },
                FromClientMessage::Answer(entry) => {
                    let answer = String::from_iter(entry);
                    let r = meval::eval_str(&answer);
                    let you_win = match r {
                        Ok(result) => {
                            println!("The answer is '{} = {}'", answer, result);
                            result == 24.0
                        }
                        _ => {
                            println!("The answer is '{}'", answer);
                            false
                        }
                    };
                    let message = if you_win {
                        FromServerMessage::TurnYouWin
                    } else {
                        FromServerMessage::TurnOtherWin
                    };

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
