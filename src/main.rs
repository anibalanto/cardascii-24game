mod common;
mod client;
mod server;
mod terminal;
mod command;
mod core_cards;
mod answer_analizer;
//mod answer_numbers;

//#[macro_use]
//extern crate serde_derive;

//#[macro_use]
//extern crate bincode;

use message_io::network::{Transport, ToRemoteAddr};

use std::net::{ToSocketAddrs};

const HELP_MSG: &str = concat!(
    "Usage: cardascii-24game table <port>\n",
    "       cardascii-24game play (<ip-table>:<port> | url)"
);

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).unwrap_or(&"".into()).as_ref() {
        "play" => match args.get(2) {
            Some(remote_addr) => {
                let remote_addr = remote_addr.to_remote_addr().unwrap();
                client::run(Transport::Ws, remote_addr);
                return;
            }
            None => (),
        },
        "table" => {
            match args.get(2).unwrap_or(&"".into()).parse() {
                Ok(port) => {
                    let addr = ("0.0.0.0", port).to_socket_addrs().unwrap().next().unwrap();
                    server::run(Transport::Ws, addr);
                    return;
                }
                Err(_) => () ,
            };
        }
        _ => (),
    }
    return println!("{HELP_MSG}");
}
