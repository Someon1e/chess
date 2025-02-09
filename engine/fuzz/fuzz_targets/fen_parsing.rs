#![no_main]

use engine::board::Board;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(string) = std::str::from_utf8(data) {
        let _board = Board::from_fen(string);
    }
});
