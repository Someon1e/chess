#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

fn clean() {
    let mut clean = Command::new("cargo").arg("clean").spawn().unwrap();
    assert!(clean.wait().unwrap().success());
}

fn build_instrument(target: &str) {
    let mut build = Command::new("cargo")
        .env(
            "RUSTFLAGS",
            "-Ctarget-cpu=native -Cprofile-generate=target/pgo-data",
        )
        .args(["build", "--release", &("--target=".to_owned() + target)])
        .spawn()
        .unwrap();
    assert!(build.wait().unwrap().success());
}

// TODO: more positions

const SEARCH_POSITIONS_QUICK: [&str; 2] = [
    "position fen 8/k7/3p4/p2P1p2/P2P1P2/8/8/K7 w - - 0 1", // Lasker-Reichhelm Position
    "position fen 8/8/2bq4/4kp2/2B5/5P1Q/8/7K w - - 11 1",  // Liburkin, 1947
];
const SEARCH_POSITIONS_SLOW: [&str; 2] = [
    "position startpos moves e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 c2c3 g8f6 d2d3 a7a6", // Italian Game: Classical Variation, Giuoco Pianissimo, with a6

    "position startpos moves g1f3 d7d5 d2d4 g8f6 c2c4 e7e6 b1c3 c7c6 e2e3 b8d7 d1c2 f8d6 f1e2 e8g8 b2b3 b7b6",
];
const PERFT_POSITION: [(&str, u8); 2] = [
    (
        "position fen r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", // Kiwipete
        4,
    ),
    (
        "position fen r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", // An alternative Perft given by Steven Edwards
        4,
    ),
];

fn run(target: &str) {
    let mut run = Command::new(format!("../engine/target/{target}/release/engine"))
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = run.stdin.take().unwrap();

    macro_rules! send {
        ($string:expr) => {
            writeln!(input, $string).unwrap();
            input.flush().unwrap();
        };
    }
    send!("uci");

    for position in SEARCH_POSITIONS_QUICK {
        send!("isready");
        send!("ucinewgame");
        send!("{position}");
        send!("go movetime 400");
    }
    for (position, depth) in PERFT_POSITION {
        send!("isready");
        send!("ucinewgame");
        send!("{position}");
        send!("go perft {depth}");
    }
    for position in SEARCH_POSITIONS_SLOW {
        send!("isready");
        send!("ucinewgame");
        send!("{position}");
        send!("go movetime 2500");
    }

    send!("quit");

    assert!(run.wait().unwrap().success());
}

fn merge_profile_data() {
    let mut optimize = Command::new("llvm-profdata")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .args([
            "merge",
            "-o",
            "target/pgo-data/merged.profdata",
            "target/pgo-data",
        ])
        .spawn()
        .unwrap();
    assert!(optimize.wait().unwrap().success());
}

fn build_optimised(target: &str) {
    let mut build = Command::new("cargo")
        .env(
            "RUSTFLAGS",
            "-Ctarget-cpu=native -Cprofile-use=target/pgo-data/merged.profdata",
        )
        .args(["build", "--release", &("--target=".to_owned() + target)])
        .spawn()
        .unwrap();
    assert!(build.wait().unwrap().success());
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let target = &args
        .get(1)
        .expect("Specify target, for example: cargo run --release x86_64-pc-windows-msvc");

    env::set_current_dir("../engine").unwrap();

    clean();
    build_instrument(target);
    run(target);
    merge_profile_data();
    build_optimised(target);
}
