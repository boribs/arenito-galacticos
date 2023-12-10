// Rust app just connects to pipe
// should panic if pipe doesn't exist
// Once it reads something from pipe, this process becomes writer.

use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
enum State {
    READER,
    WRITER,
}

const PIPE_PATH: &str = "./pipe";

fn main() {
    let mut cin = String::new();
    let mut state = State::READER;

    loop {
        println!("{:?}", state);
        match state {
            State::READER => {
                let pipe = File::open(PIPE_PATH);

                pipe
                    .as_ref()
                    .expect("something went wrong")
                    .read_to_string(&mut cin);

                if !cin.is_empty() {
                    println!("read: {}", cin);
                    cin.clear();
                }
                state = State::WRITER;
            },
            State::WRITER => {
                let pipe = File::create(PIPE_PATH);

                pipe
                    .as_ref()
                    .expect("something went wrong")
                    .write_all(b"some bytes");

                state = State::READER;
            },
        }
    }
}
