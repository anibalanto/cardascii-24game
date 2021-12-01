use std::cmp::{max, min};
use std::io;
use std::io::Write;
use std::thread;
use std::time;

use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use crate::common::{AnswerData, BYTECOUNT, HandCardData};
use crate::terminal::VisualDeck;

pub fn get_command(deck: & mut VisualDeck, hand: &HandCardData) -> Option<String> {
    // Set terminal to raw mode to allow reading stdin one key at a time
    let mut stdout = io::stdout().into_raw_mode().unwrap();

    // Use asynchronous stdin
    let mut stdin = termion::async_stdin().keys();

    let mut buffer: AnswerData = [' '; BYTECOUNT];
    let mut i : usize = 0;
    loop {
        if let Some(Ok(key)) = stdin.next() {
            match key {
                termion::event::Key::Left => {
                    i = max(0, i - 1);
                },
                termion::event::Key::Right => {
                    i = min(BYTECOUNT - 1, i + 1);
                },
                termion::event::Key::Backspace => {
                    if i > 0 {
                        i -= 1;
                        buffer[i] = ' ';
                    }
                },
                termion::event::Key::Char(char) => {
                    if char == '\n' {
                        break;
                    }

                    if i < BYTECOUNT {
                        buffer[i] = char;
                        i += 1;
                    }

                }
                _ => ()
            }
        }
        deck.draw_hand(&hand);
        write!(
            stdout,
            "{}>> {}",
            termion::cursor::Goto(2, 25),
            buffer.iter().collect::<String>()
        )
            .unwrap();
        write!( stdout, "{}", termion::cursor::Goto((5 + i) as u16, 25) ). unwrap();


        stdout.lock().flush().unwrap();

        thread::sleep(time::Duration::from_millis(50));
    }
    Some(buffer.iter().collect())
}
