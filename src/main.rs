use std::{
    io::{stdin, BufRead},
    time::Instant,
};

use chess::{
    board::{piece::Piece, square::Square, Board},
    engine::Engine,
    move_generator::move_data::{Flag, Move},
};

const START_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Max time for thinking
const MAX_TIME: u128 = 5 * 1000;

fn main() {
    let mut fen = None;
    let mut moves: Vec<String> = Vec::new();

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
                let mut index = 1;
                let mut startpos = true;
                let mut building_fen = String::new();
                while let Some(label) = args.get(index) {
                    index += 1;
                    match *label {
                        "moves" => {
                            moves.clear();
                            while let Some(uci_move) = args.get(index) {
                                moves.push((*uci_move).to_owned());
                                index += 1;
                            }
                        }
                        "fen" => {
                            startpos = false;
                        }
                        "startpos" => {
                            startpos = true;
                        }
                        _ => {
                            if !startpos {
                                building_fen.push_str(label);
                                building_fen.push(' ')
                            }
                        }
                    }
                }
                if startpos {
                    fen = Some(START_POSITION_FEN.to_owned());
                } else {
                    fen = Some(building_fen)
                }
            }
            "ucinewgame" => {}
            "go" => {
                let mut white_time = None;
                let mut black_time = None;
                let mut white_increment = None;
                let mut black_increment = None;
                let mut _moves_to_go = None;
                let mut _depth = None;
                let mut _nodes = None;
                let mut _find_mate = None;
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
                            _moves_to_go = args.get(index);
                        }
                        "depth" => {
                            index += 1;
                            _depth = args.get(index);
                        }
                        "nodes" => {
                            index += 1;
                            _nodes = args.get(index);
                        }
                        "mate" => {
                            index += 1;
                            _find_mate = args.get(index);
                        }
                        "movetime" => {
                            index += 1;
                            move_time_in_ms =
                                Some(args.get(index).unwrap().parse::<u128>().unwrap());
                        }
                        "perft" => {
                            index += 1;
                            _depth = args.get(index);
                        }
                        "infinite" => {
                            move_time_in_ms = Some(MAX_TIME);
                        }
                        _ => unimplemented!(),
                    }
                    index += 1;
                }
                let board = &mut Board::from_fen(&fen.unwrap());
                let mut engine = Engine::new(board);
                fen = None;

                for uci_move in &moves {
                    let (from, to) = (&uci_move[0..2], &uci_move[2..4]);
                    let (from, to) = (Square::from_notation(from), Square::from_notation(to));
                    let piece = engine.board().piece_at(from).unwrap();

                    let mut flag = Flag::None;
                    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
                        if (from.rank() - to.rank()).abs() == 2 {
                            flag = Flag::PawnTwoUp
                        } else if engine.board().game_state.en_passant_square == Some(to) {
                            flag = Flag::EnPassant
                        } else if let Some(promotion) = uci_move.chars().nth(4) {
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
                    } else if (piece == Piece::BlackKing || piece == Piece::WhiteKing)
                        && (from.file() - to.file()).abs() > 1
                    {
                        flag = Flag::Castle
                    }

                    engine.make_move(&Move { from, to, flag })
                }
                moves.clear();
                println!("{}", engine.board().to_fen());

                let think_time = move_time_in_ms.unwrap_or_else(|| {
                    let clock_time = (if engine.board().white_to_move {
                        white_time
                    } else {
                        black_time
                    })
                    .unwrap();
                    let increment = (if engine.board().white_to_move {
                        white_increment
                    } else {
                        black_increment
                    })
                    .unwrap_or(0);
                    (clock_time / 20 + increment / 2).min(MAX_TIME)
                });

                let search_start = Instant::now();
                let (best_move, _evaluation) = engine.iterative_deepening(
                    &mut |depth, (best_move, evaluation)| {
                        println!(
                            "info depth {depth} score cp {evaluation} time {}",
                            search_start.elapsed().as_millis()
                        )
                        // TODO: fix crash when depth goes very high
                    },
                    &mut || search_start.elapsed().as_millis() > think_time,
                );
                println!(
                    "bestmove {}{}{}",
                    best_move.from().to_notation(),
                    best_move.to().to_notation(),
                    match best_move.flag() {
                        Flag::QueenPromotion => "q",
                        Flag::RookPromotion => "r",
                        Flag::KnightPromotion => "n",
                        Flag::BishopPromotion => "b",
                        _ => "",
                    }
                )
            }
            "stop" => {}
            "quit" => return,
            _ => unimplemented!(),
        }
    }
}
