#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::io::stdin;

use core::cell::RefCell;
use engine::{
    board::Board,
    search::{transposition::megabytes_to_capacity, Search},
    uci::{SpinU16, UCIProcessor},
};

/// Max time for thinking.
const MAX_TIME: u64 = 40 * 1000;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn print_string(output: *const u8, length: u32);
}

thread_local! {
    static UCI_PROCESSOR: RefCell<UCIProcessor> = RefCell::new(UCIProcessor::new(
        MAX_TIME,

        |output: &str| {
            #[cfg(target_arch = "wasm32")]
            unsafe { print_string(output.as_ptr(), output.len() as u32) };

            #[cfg(not(target_arch = "wasm32"))]
            println!("{output}");
        },

        SpinU16::new(8..2049, 32),
    ));

    #[cfg(target_arch = "wasm32")]
    static INPUT: RefCell<String> = RefCell::new(String::new())
}

#[no_mangle]
#[cfg(target_arch = "wasm32")]
pub extern "C" fn send_input(input: u8) {
    let character = input as char;
    if character == '\n' {
        INPUT.with(|input| {
            process_input(&input.borrow());
            input.borrow_mut().clear()
        })
    } else {
        INPUT.with(|input| input.borrow_mut().push(character))
    }
}

fn process_input(input: &str) -> bool {
    let mut quit = false;
    let mut args = input.split_whitespace();
    UCI_PROCESSOR.with(|uci_processor| match args.next().expect("Empty input") {
        "isready" => uci_processor.borrow().isready(),
        "go" => uci_processor.borrow_mut().go(&mut args),
        "position" => uci_processor.borrow_mut().position(&mut args),
        "ucinewgame" => uci_processor.borrow_mut().ucinewgame(),
        "setoption" => uci_processor.borrow_mut().setoption(input),

        "uci" => uci_processor.borrow().uci(),
        "stop" => uci_processor.borrow().stop(),
        "quit" => quit = true,

        "bench" => {
            const SEARCH_POSITIONS: [&str; 2] = [
                "position fen 8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1", // Lasker-Reichhelm Position
                "position fen 8/8/2bq4/4kp2/2B5/5P1Q/8/7K w - - 11 1",  // Liburkin, 1947
            ];

            let mut search = Search::new(
                Board::from_fen(Board::START_POSITION_FEN),
                megabytes_to_capacity(32),
            );

            for position in SEARCH_POSITIONS {
                let board = Board::from_fen(position);
                search.new_board(board);
                search.clear_cache_for_new_game();
                search.clear_for_new_search();

                // TODO: actually run bench
            }
        }

        _ => panic!("Unrecognised command"),
    });
    quit
}

fn main() {
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let quit = process_input(&input);
        if quit {
            break;
        }
    }
}
