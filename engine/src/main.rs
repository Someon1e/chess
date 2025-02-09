#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::{env, io::stdin};

use core::cell::RefCell;
use engine::{
    board::Board,
    search::{transposition::megabytes_to_capacity, Search, TimeManager},
    timer::Time,
    uci::{SpinU16, UCIProcessor},
};

/// Max time for thinking.
const MAX_TIME: u64 = 40 * 1000;

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn print_string(output: *const u8, length: u32);
}

fn out(output: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        print_string(output.as_ptr(), output.len() as u32)
    };

    #[cfg(not(target_arch = "wasm32"))]
    println!("{output}");
}

thread_local! {
    static UCI_PROCESSOR: RefCell<UCIProcessor> = RefCell::new(UCIProcessor::new(
        MAX_TIME,

        |output: &str| {
            out(output);
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

fn bench() {
    /// 200 randomly chosen positions and depths from lichess-big3-resolved
    const SEARCH_POSITIONS: [(&str, u8); 200] = [
        ("6k1/7p/4p1p1/p7/3b3P/5R1K/8/8 b - - 0 1", 6),
        ("1r6/5k1p/p5p1/3p4/3Bp1P1/1P3p1P/6r1/2R2K2 w - - 0 1", 12),
        ("7k/6p1/p6p/Qp6/8/1K6/4q2P/8 b - - 1 1", 12),
        (
            "2r2rk1/1p3pb1/p2q2p1/3P4/2R5/1Q5P/P1P3B1/1K1R4 w - - 0 1",
            13,
        ),
        (
            "4r3/pb1q1r1k/1pn1p1pp/3p4/P1pP1PN1/6P1/1BP1R1BP/3QR1K1 b - - 0 1",
            3,
        ),
        ("8/8/6p1/5r1k/R6P/6K1/8/8 w - - 0 1", 4),
        ("4r2k/2q3p1/p6p/1p6/4p3/P3Q2P/1PP3P1/1K1R4 b - - 0 1", 8),
        ("4r3/p2Q1pk1/5bp1/7p/P2p4/1P1B4/K1P3r1/8 b - - 0 1", 9),
        ("7r/p3kp2/2p2Rp1/1pK1P3/1P4PP/P7/8/8 w - - 0 1", 7),
        ("2kr2nr/1p6/B3n2p/3q2p1/8/Q7/P1P2PPP/4R1K1 w - - 0 1", 9),
        ("8/p2kr2p/1pq3p1/3p1p2/7P/2Q5/PB4PK/8 w - - 0 2", 2),
        ("r5k1/pp2qppp/8/3P4/3N4/P4Q2/RP4PP/5R1K w - - 0 2", 3),
        ("R7/7k/5Kp1/3P1p1p/8/6P1/3r4/8 w - - 0 1", 12),
        ("5rk1/2p2bp1/1p1p3p/1P3P2/2P1Q3/6PP/r3P3/5RK1 w - - 0 1", 12),
        ("r3k3/pp1b2pB/2p2pNn/8/8/5N1P/P4PP1/3R2K1 w - - 0 1", 12),
        (
            "rn2kb1r/1q1p1pp1/pp2p2p/4P3/2PQNB2/P7/1P3PPP/R3R1K1 b kq - 0 2",
            13,
        ),
        ("8/3R4/p1p1nr2/N1k3p1/8/5P1P/PPP2KP1/8 b - - 0 1", 4),
        ("r7/pR2prb1/2p5/4p1kp/8/2PQ1n2/PP3P2/1K5R b - - 0 1", 5),
        ("5rk1/1R3ppp/3p4/3qp3/8/8/4K3/8 w - - 0 1", 6),
        (
            "r4k1r/p1pq1p2/6p1/1p1Pb1Bp/6n1/1BPP1QP1/PP2N1P1/RN2K2R b KQ - 0 1",
            11,
        ),
        (
            "1nkr3r/1p2b1p1/p1p1p1pp/3n2P1/1PBPN3/2P2PN1/1P1B4/R3K1R1 w - - 0 2",
            6,
        ),
        (
            "4rrk1/2p1n1b1/p3q1pp/1p1pPp2/3P4/Q2B1N2/PP3PPP/3RR1K1 w - - 0 1",
            12,
        ),
        (
            "4kr2/1p2rn2/3p1Q2/pqp2PP1/4PN2/P1PP4/1P6/1K3R1R w - - 0 1",
            4,
        ),
        ("8/5pk1/2p5/2P2KPp/3pp3/r2b4/r7/2R5 b - - 0 1", 12),
        ("3rr3/p4p1k/2p3p1/1pq4p/4P3/P6P/R3N1P1/5B1K w - - 0 3", 5),
        ("5rk1/r3pp2/p2p2p1/1p1P4/8/8/R4KPP/1R6 b - - 0 1", 7),
        (
            "r2qn2r/pp1n1pk1/3p2pp/1NpP4/5N2/1P1Q2P1/P3RPBP/R5K1 w - - 0 2",
            4,
        ),
        ("5k2/7Q/p4K2/5P2/1Pp1P3/8/8/3r4 b - - 0 1", 6),
        (
            "r2q1rk1/ppp2pb1/1n1p1np1/4p3/4P1b1/2PP1N2/PP1BBP2/R1Q1KN1R w KQ - 0 1",
            6,
        ),
        ("7k/3r2p1/4Q2p/8/8/8/3q1PK1/8 w - - 0 1", 12),
        (
            "r1bq1rk1/pp2bpp1/7p/n4PnP/8/2P1P3/P2P2B1/RNB1K1NR w KQ - 0 1",
            11,
        ),
        ("r3k2r/3pnpp1/1N3n1p/p7/Pp6/1B3P2/1P3P1P/4RRK1 b - - 0 1", 5),
        ("5r2/5p2/4p3/1p1k4/7R/2Q5/P4PP1/5K2 w - - 0 1", 11),
        (
            "r1r3k1/p4p1p/3pp1p1/4n3/5P2/NB5P/PPP3P1/3R2K1 b - - 0 1",
            10,
        ),
        (
            "2r2rk1/1bqpbpnp/p3p1p1/6P1/1p2P1BP/4BP2/PPPQN3/2KR3R w - - 0 1",
            5,
        ),
        ("8/4p3/7p/4kpp1/1K6/4PPP1/7P/8 w - - 0 1", 5),
        ("2r2k2/1b3ppp/8/pp2pNP1/4P2P/P2R4/1PP5/1K6 w - - 0 1", 4),
        (
            "rn3r2/2pq3k/p6p/1pbbPpBQ/7P/1B1P4/PPP3P1/RN3R1K b - - 0 1",
            5,
        ),
        (
            "2rq1rk1/4bppp/1n2p3/3pP3/2pP4/2P2N1P/2QN1PP1/R4RK1 b - - 0 1",
            10,
        ),
        (
            "r4rk1/pp1q1ppb/2p2n1p/3p4/2BpP1P1/2NP1Q1P/PPP2PK1/R4R2 w - - 0 1",
            7,
        ),
        (
            "2q2rk1/1p3ppp/p2R1b2/8/P1r1P3/2N2Q1P/1P3PP1/2R3K1 b - - 0 1",
            2,
        ),
        ("1n3r2/r4kpp/b3p3/5p2/1b2P3/5NPB/N4P1P/2RR2K1 b - - 0 3", 1),
        ("8/8/8/6PB/8/r1pk1P2/8/3K4 w - - 0 1", 1),
        (
            "r4r1k/pp4b1/2p2np1/3q1p2/3P4/3Q4/PPP2PP1/4RRK1 w - - 0 1",
            6,
        ),
        (
            "rnb1r2k/pp2p1bp/6p1/1P3p2/2BN4/B7/2P1QPPP/3RR1K1 b - - 0 1",
            4,
        ),
        ("3r4/6r1/2R2p2/p4p2/P4p2/k3PKP1/8/1R6 w - - 0 1", 6),
        (
            "3r1rk1/p1p2pb1/1p4pp/8/4b2Q/2P1q1PN/PP4P1/4RR1K b - - 0 1",
            3,
        ),
        (
            "2kr1b1r/2p1p3/p1p5/6pp/3P1p2/2P5/PP1B1PPP/R4RK1 b - - 0 1",
            1,
        ),
        (
            "1r1q1rk1/QP3pp1/3p4/P2Np1np/4P1b1/1B1P2P1/2P2P1P/R6K b - - 0 1",
            3,
        ),
        (
            "r1b1k2r/1pq1bppp/p2ppn2/6B1/3NPP2/1PN5/PP1Q2PP/2KR3R b kq - 0 1",
            1,
        ),
        ("N7/pp2nB1p/3p4/2p5/k3P3/5K2/5P2/2q5 b - - 1 1", 9),
        ("5k2/4p2Q/p2p4/2p3p1/8/2P5/P5PK/4r3 w - - 0 1", 13),
        ("8/5pk1/2R3pb/1p5p/3P4/2P5/1r6/R4K2 w - - 0 1", 9),
        (
            "r1b1k2r/ppp2ppp/2n2n2/8/8/2B2N2/PPq1BPPP/R3KR2 w Qkq - 0 1",
            8,
        ),
        (
            "r3r1k1/5ppp/pp3q2/8/1PP1b1P1/P3B2P/5P2/R2QR1K1 b - - 0 1",
            4,
        ),
        (
            "Q1bk1bnr/p1p2q1p/3p1pp1/p2B4/3P4/2P1PN2/1P3PPP/RN2K2R b KQ - 0 1",
            3,
        ),
        ("8/8/4p3/4P1Pp/R4P2/8/1k1K4/6r1 b - - 0 1", 7),
        ("5k2/8/b7/8/7P/6P1/BK6/8 b - - 0 1", 2),
        ("8/8/Q7/8/2r5/3kn1P1/7K/8 b - - 0 1", 9),
        ("8/5kp1/8/5P2/7r/2K5/8/6R1 b - - 0 1", 13),
        (
            "r5k1/p3b2p/1p2n1p1/1P1rPp2/P1pP1P2/2P1B3/7P/R3R1K1 b - - 0 1",
            10,
        ),
        ("3r1r2/6pk/1pBpbp1p/p1p5/P1P5/1P2P2P/5PPK/R2R4 w - - 0 1", 3),
        (
            "r6r/pp1bkpbp/3ppnp1/2p3N1/2N1P3/2P5/PP1P1PPP/R1B1K2R w KQ - 0 1",
            3,
        ),
        ("2kr1b2/ppq2pp1/8/4P3/7r/4B1N1/PPP3QP/R5K1 w - - 0 1", 10),
        ("6k1/p5pp/1p2q3/2p2p2/P1Pp4/3P4/1P3PPP/5K2 w - - 0 1", 9),
        ("r7/pppkbNpp/8/8/3p4/8/PPP2PPP/RN4K1 b - - 0 1", 5),
        ("5rk1/4qp2/p1n1p1pQ/4n3/2PpN1PB/3P3P/P5K1/R4R2 b - - 0 1", 1),
        ("r6r/p2p1kpp/5n2/8/1pPR4/1P6/P4PPP/R1B3K1 b - - 0 2", 8),
        ("6k1/4Q3/4R3/7p/3N4/r6P/5PP1/6K1 b - - 0 1", 9),
        (
            "2kr1b1r/1pp1pp2/p2p1np1/3P3p/1PP1PP2/2N3P1/P6P/R1B1R1K1 w - - 0 1",
            9,
        ),
        ("6rk/3r4/8/7p/8/1P4Q1/5PbP/6K1 w - - 0 1", 7),
        (
            "2rq1rk1/p4pbp/1p1p2p1/4p2n/1p1nP3/2NP2PP/PBP2PBK/R2QR3 w - - 0 1",
            10,
        ),
        ("8/7N/1P6/4p2B/2k2p2/4nP2/8/7K b - - 0 1", 1),
        ("6k1/6p1/5pb1/1p2pN2/1b1pPnQP/3P1NK1/1rr5/5B2 w - - 0 1", 9),
        ("6k1/p4p2/1p4p1/4B2p/3P3K/1Pq5/P7/8 b - - 0 1", 7),
        (
            "r2q4/p1p1k3/2p1pp2/6pr/1b1P4/2N1PNP1/PP2QPP1/R3K3 w Q - 0 5",
            3,
        ),
        ("8/p1k3n1/1pbp4/4pP2/4p2P/2P3BP/P5BK/5R2 w - - 0 1", 9),
        ("8/8/4pk2/P3p3/4P3/2r2PP1/R5KP/8 w - - 0 2", 13),
        ("8/6pp/3k1p2/4r3/3N4/5K2/5PPP/8 w - - 0 1", 7),
        (
            "r4rk1/2p2pp1/p1n1p2p/1p1p4/7q/1PNP1P2/PBPN4/R3QK2 b - - 0 1",
            1,
        ),
        ("4k3/6Rp/2p5/2P2P2/8/1K2n2P/8/3R4 w - - 0 2", 6),
        ("5k2/1p4pp/p1pb4/8/1P1P4/7P/1PPB2K1/4r3 b - - 0 1", 4),
        (
            "r3kb1r/pp3pp1/2bp1n1p/q1p1pP2/4P3/1PPP1N2/PB1NQ1PP/R4R1K b kq - 0 1",
            12,
        ),
        (
            "r2q1rk1/pb3ppp/1p1bp3/8/1n1P4/1PnB1N2/PB1N1PPP/3Q1RK1 w - - 0 1",
            5,
        ),
        ("8/1p4pk/1N5p/4R3/8/7P/3n1PPK/r7 w - - 0 1", 3),
        (
            "5rk1/b1r1q1p1/p2p3p/Pp1Pp3/1Pp2P1P/B1P3P1/3Q2K1/1R6 w - - 0 1",
            3,
        ),
        ("7k/1N1r1p1p/p5p1/4p3/1P6/6P1/4KP1P/8 w - - 0 1", 2),
        ("6k1/8/b1q2p2/6p1/pB2P3/P4PK1/8/7Q b - - 0 1", 13),
        ("6k1/1R5p/8/8/1P2P2r/4KP2/8/8 w - - 0 1", 13),
        (
            "1q1n2k1/4b1pp/3p1n2/1N1Pp3/1P2Pp2/3Q1P2/4N1PP/R5K1 w - - 0 2",
            8,
        ),
        (
            "2r2rk1/ppqnbppp/2pp1n2/4p3/3PP3/1QPBBN1P/PP3PP1/R4RK1 w - - 0 1",
            12,
        ),
        ("8/7k/8/7p/5Q1P/q5P1/3K4/8 b - - 0 1", 11),
        ("4k3/1p3p2/2n4p/8/3n1P1P/8/6P1/3R2K1 b - - 0 1", 2),
        ("8/3Q4/6pk/3K4/8/8/PP6/8 w - - 0 1", 5),
        (
            "r3k2r/ppq2ppp/2pb2b1/4p3/B4B2/2P2P2/PPP3PP/R2QR1K1 b kq - 0 1",
            3,
        ),
        ("6k1/3R3p/1b4p1/p7/P7/1P3R1P/1r3PP1/5K2 b - - 0 1", 11),
        (
            "r3r1k1/1pp2ppp/p1n2q2/2P5/1P2p3/P3P3/4QPPP/RN2NRK1 w - - 0 2",
            6,
        ),
        (
            "r2q1rk1/4bpp1/2n1p1p1/p2p3N/P2P1B1P/1B6/1p3PP1/R4RK1 w - - 0 3",
            10,
        ),
        ("2B2rk1/3R1ppp/1p2p1b1/2P1P3/8/2p5/P5PP/6K1 w - - 0 3", 10),
        ("5k2/1bp4p/pp1p2pN/6P1/1bP2pP1/4rP2/P7/3KNR2 w - - 0 1", 12),
        ("3Q4/2K4p/8/4p2k/4P3/8/8/8 w - - 0 1", 1),
        ("8/5Q2/1kBp1p2/3P4/3bp3/2n5/8/K7 w - - 0 1", 12),
        ("8/8/1k3p2/1p2bK2/8/8/n7/8 w - - 0 1", 13),
        (
            "5rk1/3q1ppp/3p1b2/1PpPp1n1/r3P1P1/PN4P1/1Q1NB1KP/R1R5 w - - 0 1",
            11,
        ),
        ("8/8/8/5kp1/K7/P7/3Q1P2/7q b - - 0 1", 5),
        ("2k5/ppp5/5p2/5P2/8/4n2P/PPPN2q1/1K5R w - - 0 5", 8),
        ("8/R7/5K2/5p2/5r2/1k4n1/8/8 w - - 0 1", 5),
        ("r1b2k2/1p2bp2/4p2p/p1q5/8/2N1PB2/PP3PRP/2KQ4 w - - 0 1", 7),
        ("3Q4/8/3p4/3Pb2p/4Kp2/8/R6r/3B2k1 w - - 0 1", 6),
        ("7k/p5bp/1p4p1/4p3/8/5P1P/P2r1P2/2R2RK1 w - - 0 5", 10),
        ("6k1/5p2/1n1P2p1/1P1pP1P1/2p5/2B2K2/8/8 w - - 0 1", 8),
        (
            "2r2rk1/1p3ppp/p4n2/3p4/2PP4/1P1p2P1/q2N1PP1/2R2RK1 w - - 0 1",
            2,
        ),
        ("8/p7/1p5p/6q1/3K4/8/2k5/8 b - - 0 1", 8),
        (
            "r2qk2r/p3bpp1/2pp2bp/8/P2pn3/2P3QP/1P2N1P1/RNB2RK1 w kq - 0 1",
            9,
        ),
        ("r3q1k1/ppp2r1p/5ppQ/8/3nP3/B7/P4PPP/2R1R1K1 b - - 0 1", 8),
        ("8/1p5k/p2p4/2p5/2P5/3P1R2/Pb6/4rB1K w - - 0 1", 2),
        ("4k3/R7/4p2p/1r2n2P/5p2/1pP2P2/8/3K4 w - - 0 1", 10),
        (
            "r3r1k1/1bpnqpbp/1p2p1p1/p2P4/2PP1P2/P1NBQNPP/1P4K1/4RR2 w - - 0 1",
            3,
        ),
        ("3r3k/5P1p/p5p1/1p2n3/1P6/5bNP/2q1N1PK/4R1Q1 b - - 0 2", 8),
        ("8/2R5/4k2p/P4pp1/3B4/6r1/8/4K3 w - - 0 1", 3),
        ("8/5p2/1ppp3p/p1k2p2/5P2/1PbP1NP1/P4K1P/8 w - - 0 1", 4),
        ("5rk1/p4p1p/8/3RNb2/8/P3P3/1P4r1/K2R4 b - - 0 1", 10),
        (
            "4b1R1/ppp1r3/1b4p1/3P1p2/2P1p3/1P4P1/P2k4/4NK2 w - - 0 1",
            1,
        ),
        ("5r1k/6p1/p3P1R1/1p6/1P1n3P/P6K/8/8 b - - 0 3", 2),
        ("5k2/p3rp1p/4p1p1/8/3P1B2/2r3P1/P3PPKP/R7 b - - 0 2", 12),
        ("8/8/PR2b3/4k1p1/2r3P1/5P1K/8/8 b - - 0 1", 7),
        ("6k1/p5p1/1pr1P1B1/1b1p2p1/3P4/P1P1RPK1/7P/8 b - - 0 1", 7),
        (
            "8/2q1pk2/1r4p1/p1p2b1p/P2p1PnP/1P1P1R2/2P1Q1P1/6K1 w - - 0 1",
            7,
        ),
        ("R7/5pk1/6p1/4p1P1/8/8/4KP1r/8 w - - 0 1", 13),
        ("2RR1b1r/pk4p1/1p5p/8/5p2/5NP1/P3rP1P/6K1 w - - 0 1", 1),
        ("8/8/6p1/2P1k2p/4b2P/PK4p1/3R4/8 w - - 0 1", 8),
        ("1r6/2n1Q3/2k1p3/4q3/1pP5/2p1B3/2B2P2/R4K2 w - - 0 1", 13),
        ("r7/7R/P4p2/1K6/6P1/6kP/8/8 b - - 0 1", 2),
        ("5rk1/7p/3p2p1/RP1Pp3/6P1/1B3n1K/8/Bb1N4 b - - 0 1", 5),
        ("3r1rk1/5pp1/3p3p/p3p3/1PPn1Q2/5P2/B5PP/3R1RK1 w - - 0 1", 1),
        ("3r3r/ppk2p1p/5nbp/3P4/2PN2PP/2P5/P3B3/R3K2R w KQ - 0 1", 2),
        ("8/8/8/6PP/7K/5k2/4r3/R7 b - - 0 1", 3),
        ("5r2/p6k/4pn2/8/6P1/P1P4P/1P1R4/2K5 w - - 0 1", 13),
        (
            "r4rk1/1pp2ppp/p1p5/6q1/N2Pn3/6P1/PPP2P1P/R4QK1 w - - 0 3",
            4,
        ),
        ("2r2rk1/pp3pp1/7p/4p3/8/4QpRP/PqB2P2/2R3K1 w - - 0 1", 9),
        (
            "r1b2rk1/p3ppbp/n1B2np1/4N3/1qpP4/1PN3P1/PB3P1P/R2Q1RK1 b - - 0 1",
            12,
        ),
        (
            "r3k2r/2p1qpp1/5n2/3p3p/Nn1PpP2/1PN1P2P/1KPQ2P1/3R1R2 b kq - 0 1",
            7,
        ),
        ("8/p4ppk/7p/1P6/4r1P1/P2p3P/5P1K/2r5 b - - 0 1", 12),
        ("4r2k/ppq3pp/2p2n2/5B2/3pP3/1P4P1/P3QP1P/2R3K1 w - - 0 2", 6),
        (
            "r2q1rk1/pppnbp2/5n1Q/3p1b2/3P4/2N2N1P/PPP1BPP1/2KR3R b - - 0 1",
            10,
        ),
        (
            "6k1/p1q1br1p/1p2b3/2p1PppP/8/2B2BP1/PPPQ2K1/3R4 b - - 0 1",
            13,
        ),
        (
            "4nrk1/1q1p1ppp/ppr1p3/8/1bP5/1PN1PQP1/PB1R1P1P/R5K1 w - - 0 2",
            5,
        ),
        ("8/5p2/8/8/8/7R/p3r3/2k3K1 w - - 0 1", 7),
        (
            "rnb1kb1r/ppp3pp/3p2n1/3qp1B1/8/2PP2QB/PP2PP1P/RN2K1NR w KQkq - 0 1",
            6,
        ),
        ("7r/3r1p2/1p3pk1/4p3/p5P1/P2RPKN1/1P3P2/3R4 b - - 0 1", 12),
        (
            "3q2k1/pb1r1pb1/3n2pp/1p1P4/5B2/1B3N1P/P3QPPK/4R3 b - - 0 1",
            9,
        ),
        ("4N3/8/p7/6R1/4k1P1/7P/8/r4nK1 b - - 0 1", 10),
        ("8/p7/K7/3B4/1P6/P5k1/7p/8 b - - 0 1", 11),
        (
            "2k4r/ppp2ppp/5n2/2B3q1/2P5/1P2P3/P4PPP/RN1bKB1R w KQ - 0 1",
            2,
        ),
        (
            "6k1/p3npp1/1p3n1p/1q6/3p3Q/b2P2P1/2r1PPBP/2NR2K1 w - - 0 1",
            7,
        ),
        (
            "1k1r1r2/pb1p4/1pnNp2p/1R2Pp2/5p2/1RP2B2/q1PK1QPP/8 b - - 0 1",
            6,
        ),
        ("6k1/5p1p/p2b2p1/1p1p4/1P1P2P1/P3B2P/2q2PK1/8 w - - 0 1", 11),
        (
            "1rr2nk1/p1q1ppbp/2bp2p1/2p5/4PB2/1PP2NP1/P1Q2PBP/1R2R1K1 w - - 0 1",
            2,
        ),
        ("5n2/5k2/2p1p3/1bPpP1p1/p5Pp/P1P1BK2/2B4P/8 b - - 0 1", 5),
        (
            "2k4r/2nq1p2/2p1bp2/1pPp4/4pP2/4P3/1KP1Q1B1/3R2N1 b - - 0 1",
            1,
        ),
        (
            "r2q1rk1/pp3p2/2p1p1pp/4P3/3PRQ2/4R2P/P2b1PP1/6K1 w - - 0 1",
            9,
        ),
        ("8/5k2/p1p5/1pP3K1/1P6/P7/7P/8 b - - 0 1", 5),
        ("8/8/4K1p1/p6p/P2k1P1P/8/8/8 b - - 0 1", 10),
        (
            "1b2r1k1/1p3pp1/1P3np1/8/2NPp3/4B2P/1Q2KPP1/6q1 w - - 0 1",
            12,
        ),
        (
            "8/1p2k3/r1p1p2p/3p2p1/pP2P1P1/3P3P/P1PNN1B1/5RK1 w - - 0 1",
            6,
        ),
        ("r5k1/ppp4p/5p2/3n4/8/1PP5/PK2R2P/8 w - - 0 1", 11),
        (
            "rn2r1k1/1pp1qp1p/p3bp2/3np3/2B5/P1PP1N1P/1P3PP1/R1BQR1K1 w - - 0 2",
            9,
        ),
        ("2kr1bnr/pppq3p/3p4/8/3P4/4BQ2/PP3PPP/RN3RK1 b - - 0 1", 1),
        ("Q7/5k1p/8/2B1n3/2PK4/8/8/8 b - - 0 1", 4),
        ("8/6k1/3r4/1R3R2/1pr4P/6PK/8/8 b - - 0 1", 13),
        (
            "5r2/2r1qpkp/p1nnp1p1/3p4/N7/1P2P1P1/P2Q1PBP/2R2RK1 w - - 0 1",
            9,
        ),
        ("5k2/8/8/8/8/7P/6PK/3b4 w - - 0 2", 5),
        ("2r5/5pk1/R3p1p1/Nn5p/r6P/5PP1/3P1PK1/3R4 b - - 0 1", 4),
        ("8/8/4p3/5rkp/6nR/5PP1/6K1/8 b - - 0 1", 6),
        (
            "2r3k1/p3q1pp/b1r2pn1/4p3/2P1P3/b3QNP1/5PBP/R2N2K1 b - - 0 1",
            1,
        ),
        (
            "r2q2k1/pb2b1pp/1p2p3/2p1P3/2PPp3/4P1P1/PB4BP/R4QK1 w - - 0 1",
            9,
        ),
        ("3k4/7R/1r2p1p1/8/4KP2/6P1/7P/8 b - - 0 1", 1),
        ("5r1k/Q4pp1/8/3b1p2/3P4/4P1Pp/Pr1B1P1P/R2R2K1 b - - 0 1", 12),
        ("5rk1/pp4p1/2b4p/2P1qp2/1P6/P7/6PP/5R1K w - - 0 1", 2),
        ("r4rk1/2p5/1p1p3p/3P1qp1/N1P2p2/PP3Q2/7N/4R2K w - - 0 1", 2),
        ("4r3/p5pp/2k2p2/3Np3/1PP1n3/4P3/P5PP/R5K1 w - - 1 1", 1),
        (
            "r1b1k2r/pp4pp/2p2p2/2Pp4/6q1/2N1PN2/PPB3P1/R1B2RK1 w kq - 0 1",
            9,
        ),
        ("r5k1/pp3p1p/2pRb1p1/4p3/4P3/P5P1/5PBP/6K1 b - - 0 1", 6),
        ("5k2/p3n3/1p2p2p/3pP3/2pP2Pp/P1P4P/2P1B3/6K1 w - - 0 1", 9),
        ("2k5/8/5n2/2pp4/5K2/5P2/8/8 b - - 0 1", 11),
        (
            "r6r/ppp2k2/3q2p1/5p2/1P2pn2/P1Pb2Q1/1B1P2PP/R3R1K1 w - - 0 1",
            7,
        ),
        (
            "r4rk1/2p2p1p/pp6/3PP1p1/2b2p2/2N4P/PP4P1/2R3K1 w - - 0 1",
            4,
        ),
        ("8/8/1R6/6Q1/2k2pP1/5K2/P7/8 b - - 0 1", 4),
        (
            "2rqr1k1/1b1nbppp/pp1ppn2/8/2PNP3/2N1BP2/PP1Q2PP/2RR1BK1 b - - 0 1",
            6,
        ),
        ("5R2/3p1Qpk/1r5p/p1N1P3/7q/1P6/P1P3PP/7K b - - 0 1", 10),
        (
            "1r2r1k1/p2bpp1p/2pp1npQ/4qP2/4P3/2NB1R2/PPP3PP/1R5K b - - 0 2",
            6,
        ),
        ("8/6p1/4pk2/8/4PP1P/8/6K1/R7 b - - 0 1", 1),
        (
            "1r1q1rk1/pb2bpp1/1p2p2p/3pP3/3P4/2PQ1N2/PP4PP/R1B2RK1 w - - 0 1",
            12,
        ),
        ("8/1prbk1p1/p2npp1p/8/3N4/2P1PB1P/PP3KP1/3R4 w - - 0 1", 12),
        (
            "rnb1r1k1/ppppqpp1/5n2/2b5/2B1PB1Q/3P4/PPP3PP/RN2K2R b KQ - 0 1",
            6,
        ),
        ("2r5/p4pkp/1p4p1/8/B2p4/8/PP3PPP/RN2K2R w KQ - 0 1", 9),
        ("5k2/R7/7p/4pp2/3r3P/5PK1/6P1/8 w - - 0 1", 11),
        (
            "r4rk1/1p2ppbp/p3b1p1/n7/8/4PN2/PP2BPPP/R1B1R1K1 b - - 0 1",
            3,
        ),
        (
            "r3k2r/pp2pp1p/5np1/4b3/3q4/2N1BQ2/PPP3PP/R4RK1 b kq - 0 1",
            9,
        ),
        (
            "2r3qr/1b1k1p2/p1n2n1p/4pP2/P1Bp4/RP3P2/2PP3P/3QK1NR b K - 0 1",
            10,
        ),
    ];

    let mut search = Search::new(
        Board::from_fen(Board::START_POSITION_FEN),
        megabytes_to_capacity(32),
        #[cfg(feature = "spsa")]
        UCI_PROCESSOR.with(|uci_processor| uci_processor.borrow().tunables),
    );

    let mut total_nodes: u64 = 0;
    let time = Time::now();
    for (position, depth) in SEARCH_POSITIONS {
        let board = Board::from_fen(position);
        search.new_board(board);
        search.clear_cache_for_new_game();
        search.clear_for_new_search();

        let time_manager = TimeManager::depth_limited(depth);
        let result = search.iterative_deepening(&time_manager, &mut |_| {});
        out(&format!(
            "{position} {depth} {}",
            search.quiescence_call_count()
        ));
        total_nodes += u64::from(search.quiescence_call_count());
    }
    out(&format!(
        "{total_nodes} nodes {nodes_per_second} nps",
        nodes_per_second = (total_nodes * 1000) / time.milliseconds()
    ));
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
            bench();
        }

        _ => panic!("Unrecognised command"),
    });
    quit
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let args: Vec<String> = env::args().collect();

        let target = args.get(1);
        if target.is_some_and(|arg| arg == "bench") {
            bench();
            return;
        }
    }

    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let quit = process_input(&input);
        if quit {
            break;
        }
    }
}
