use std::io::{stdin, BufRead};

use core::cell::RefCell;
use engine::uci::UCIProcessor;

// Max time for thinking
const MAX_TIME: u128 = 5 * 1000;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn print_char(c: u8);
}

#[cfg(target_arch = "wasm32")]
fn print(output: &str) {
    for &c in output.as_bytes() {
        unsafe { print_char(c) }
    }
    unsafe { print_char('\n' as u8) }
}

thread_local! {
    static UCI_PROCESSOR: RefCell<UCIProcessor> = RefCell::new(UCIProcessor {
        moves: Vec::new(),
        fen: None,
        max_thinking_time: MAX_TIME,

        out: |output: &str| {
            #[cfg(target_arch = "wasm32")]
            print(output);

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

fn process_input(input: &str) {
    let mut args = input.split_whitespace();
    UCI_PROCESSOR.with(|uci_processor| match args.next().unwrap() {
        "quit" => panic!(),

        "uci" => uci_processor.borrow().uci(),
        "isready" => uci_processor.borrow().isready(),
        "position" => uci_processor.borrow_mut().position(&mut args),
        "ucinewgame" => uci_processor.borrow().ucinewgame(),
        "go" => uci_processor.borrow_mut().go(&mut args),
        "stop" => uci_processor.borrow().stop(),
        _ => unimplemented!(),
    });
}

fn main() {
    let mut stdin = stdin().lock();

    loop {
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        process_input(&input);
    }
}
