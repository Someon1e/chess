use std::{
    io::{stdin, BufRead, Write},
    time::Instant,
};

use chess::{
    board::{piece::Piece, square::Square, Board},
    engine::Engine,
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
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
                let mut move_time_in_ms = None;
                let mut index = 1;
                while let Some(label) = args.get(index) {
                    match *label {
                        "searchmoves" => {}
                        "ponder" => {}
                        "wtime" => {
                            index += 1;
                            white_time = Some(args.get(index).unwrap().parse::<u128>().unwrap());
                        }
                        "btime" => {
                            index += 1;
                            black_time = Some(args.get(index).unwrap().parse::<u128>().unwrap());
                        }
                        "winc" => {
                            index += 1;
                            white_increment =
                                Some(args.get(index).unwrap().parse::<u128>().unwrap());
                        }
                        "binc" => {
                            index += 1;
                            black_increment =
                                Some(args.get(index).unwrap().parse::<u128>().unwrap());
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
                            move_time_in_ms =
                                Some(args.get(index).unwrap().parse::<u128>().unwrap());
                        }
                        "perft" => {
                            index += 1;
                            depth = args.get(index);
                        }
                        "infinite" => {
                            move_time_in_ms = None;
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

                    let mut flag = Flag::None;
                    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
                        if let Some(promotion) = uci_move.chars().nth(4) {
                            flag = match promotion {
                                'q' => Flag::QueenPromotion,
                                'r' => Flag::RookPromotion,
                                'n' => Flag::KnightPromotion,
                                'b' => Flag::BishopPromotion,
                                _ => {
                                    panic!("Invalid promotion notation")
                                }
                            }
                        }
                    } else if piece == Piece::BlackKing || piece == Piece::WhiteKing {
                        if (from.file() - to.file()).abs() > 1 {
                            flag = Flag::Castle
                        }
                    } else if piece == Piece::BlackPawn || piece == Piece::WhitePawn {
                        if board.game_state.en_passant_square == Some(to) {
                            flag = Flag::EnPassant
                        }
                    }

                    board.make_move(&Move::with_flag(from, to, flag))
                }
                moves.clear();

                let think_time = move_time_in_ms.unwrap_or_else(|| {
                    let clock_time = (if board.white_to_move {
                        white_time
                    } else {
                        black_time
                    })
                    .unwrap();
                    (clock_time * 4 / 100).min(5 * 1000) // Use 4% of clock time, but don't use more than 5 seconds.
                                                         // TODO: take into account increment
                });

                let engine = &mut Engine::new(&mut board);
                let search_start = Instant::now();
                let (best_move, _evaluation) = engine
                    .iterative_deepening(&mut |depth, (_best_move, _evaluation)| {}, &mut || {
                        search_start.elapsed().as_millis() > think_time
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
