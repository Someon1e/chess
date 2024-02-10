use std::io::{stdin, BufRead};

use chess::uci::UCIProcessor;

// Max time for thinking
const MAX_TIME: u128 = 5 * 1000;

fn main() {
    let mut stdin = stdin().lock();

    let mut uci_processor = UCIProcessor {
        moves: Vec::new(),
        fen: None,
        max_thinking_time: MAX_TIME,
    };
    loop {
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let mut args = input.split_whitespace();
        match args.next().unwrap() {
            "quit" => return,

            "uci" => uci_processor.uci(),
            "isready" => uci_processor.isready(),
            "position" => uci_processor.position(&mut args),
            "ucinewgame" => uci_processor.ucinewgame(),
            "go" => uci_processor.go(&mut args),
            "stop" => uci_processor.stop(),
            _ => unimplemented!(),
        }
    }
}
