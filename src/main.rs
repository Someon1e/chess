use std::{
    io::{stdin, BufRead, Write},
    time::Instant,
};

use chess::{
    board::{piece::Piece, square::Square, Board},
    engine::Engine,
    move_generator::{move_data::Move, MoveGenerator},
};

const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn main() {
    let mut fen = None;
    let mut moves: Vec<String> = Vec::new();
    let mut board;

    let mut stdin = stdin().lock();

    loop {
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let args: Vec<&str> = input.split_whitespace().collect();
        match args[0] {
            "uci" => {
                println!("id name chess");
                println!("id author someone");
                println!("uciok")
            }
            "isready" => {
                println!("readyok")
            }
            "position" => {
                let position = args[1];
                fen = Some(if position == "startpos" {
                    START_POSITION_FEN.to_owned()
                } else {
                    position.to_owned()
                });
                if let Some(arg) = args.get(2) {
                    if *arg == "moves" {
                        moves.clear();
                        for uci_move in &args[3..] {
                            moves.push((*uci_move).to_owned())
                        }
                    }
                }
            }
            "ucinewgame" => {}
            "go" => {
                let mut white_time = None;
                let mut black_time = None;
                let mut white_increment = None;
                let mut black_increment = None;
                let mut moves_to_go = None;
                let mut depth = None;
                let mut nodes = None;
                let mut find_mate = None;
                let mut move_time = None;
                let mut index = 1;
                while let Some(label) = args.get(index) {
                    println!("{label}");
                    match *label {
                        "searchmoves" => {}
                        "ponder" => {}
                        "wtime" => {
                            index += 1;
                            white_time = args.get(index);
                        }
                        "btime" => {
                            index += 1;
                            black_time = args.get(index);
                        }
                        "winc" => {
                            index += 1;
                            white_increment = args.get(index);
                        }
                        "binc" => {
                            index += 1;
                            black_increment = args.get(index);
                        }
                        "movestogo" => {
                            index += 1;
                            moves_to_go = args.get(index);
                        }
                        "depth" => {
                            index += 1;
                            depth = args.get(index);
                        }
                        "nodes" => {
                            index += 1;
                            nodes = args.get(index);
                        }
                        "mate" => {
                            index += 1;
                            find_mate = args.get(index);
                        }
                        "movetime" => {
                            index += 1;
                            move_time = args.get(index);
                        }
                        "perft" => {
                            index += 1;
                            depth = args.get(index);
                        }
                        "infinite" => {
                            move_time = None;
                        }
                        _ => unimplemented!(),
                    }
                    index += 1;
                }
                board = Board::from_fen(&fen.unwrap());
                fen = None;
                for uci_move in &moves {
                    let (from, to) = (&uci_move[0..2], &uci_move[2..4]);
                    let (from, to) = (Square::from_notation(from), Square::from_notation(to));
                    let piece = board.piece_at(from).unwrap();

                    // TODO: castling, en passant
                    let move_data;
                    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
                        if let Some(promotion) = uci_move.chars().nth(4) {
                            println!("{promotion}")
                        }
                    }

                    move_data = Move::new(from, to);
                    board.make_move(&move_data)
                }
                moves.clear();

                let move_generator = &mut MoveGenerator::new(&mut board);
                let engine = &mut Engine::new(move_generator);
                let search_start = Instant::now();
                let (best_move, evaluation) = engine
                    .iterative_deepening(&mut |depth, (_best_move, _evaluation)| {}, &mut || {
                        search_start.elapsed().as_millis() > 5 * 1000
                    });
                if !best_move.is_none() {
                    println!(
                        "bestmove {}{}",
                        best_move.from().to_notation(),
                        best_move.to().to_notation()
                    )
                }
            }
            "stop" => {}
            "quit" => return,
            _ => unimplemented!(),
        }
    }
}
