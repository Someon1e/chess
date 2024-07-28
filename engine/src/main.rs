#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::io::stdin;

use core::cell::RefCell;
use engine::uci::UCIProcessor;

/// Max time for thinking.
const MAX_TIME: u64 = 40 * 1000;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn print_string(output: *const u8, length: u32);
}

thread_local! {
    static UCI_PROCESSOR: RefCell<UCIProcessor> = RefCell::new(UCIProcessor {
        moves: Vec::new(),
        fen: None,

        search: None,
        max_thinking_time: MAX_TIME,

        out: |output: &str| {
            #[cfg(target_arch = "wasm32")]
            unsafe { print_string(output.as_ptr(), output.len() as u32) };

            #[cfg(not(target_arch = "wasm32"))]
            println!("{output}");
        }
    });

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
        "quit" => quit = true,

        "uci" => uci_processor.borrow().uci(),
        "isready" => uci_processor.borrow().isready(),
        "position" => uci_processor.borrow_mut().position(&mut args),
        "ucinewgame" => uci_processor.borrow_mut().ucinewgame(),
        "go" => uci_processor.borrow_mut().go(&mut args),
        "stop" => uci_processor.borrow().stop(),
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
