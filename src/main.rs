use std::{
    io::{stdin, BufRead},
    time::Instant,
};

use chess::{
    board::{piece::Piece, square::Square, Board},
    engine::Engine,
    move_generator::move_data::{Flag, Move},
    perft::perft_root,
    uci,
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

        let mut args = input.split_whitespace();
        match args.next().unwrap() {
            "uci" => {
                println!("id name chess");
                println!("id author someone");
                println!("uciok")
            }
            "isready" => {
                println!("readyok")
            }
            "position" => {
                let mut startpos = true;
                let mut building_fen = String::new();
                while let Some(label) = args.next() {
                    match label {
                        "moves" => {
                            moves.clear();
                            while let Some(uci_move) = args.next() {
                                moves.push((*uci_move).to_owned());
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
                let mut perft = false;
                let mut depth = None;
                let mut _nodes = None;
                let mut _find_mate = None;
                let mut move_time_in_ms = None;
                while let Some(label) = args.next() {
                    match label {
                        "searchmoves" => {}
                        "ponder" => {}
                        "wtime" => {
                            white_time = Some(args.next().unwrap().parse::<u128>().unwrap());
                        }
                        "btime" => {
                            black_time = Some(args.next().unwrap().parse::<u128>().unwrap());
                        }
                        "winc" => {
                            white_increment =
                                Some(args.next().unwrap().parse::<u128>().unwrap());
                        }
                        "binc" => {
                            black_increment =
                                Some(args.next().unwrap().parse::<u128>().unwrap());
                        }
                        "movestogo" => {
                            _moves_to_go = args.next();
                        }
                        "depth" => {
                            depth = Some(args.next().unwrap().parse::<u16>().unwrap());
                        }
                        "nodes" => {
                            _nodes = args.next();
                        }
                        "mate" => {
                            _find_mate = args.next();
                        }
                        "movetime" => {
                            move_time_in_ms =
                                Some(args.next().unwrap().parse::<u128>().unwrap());
                        }
                        "perft" => {
                            perft = true;
                            depth = Some(args.next().unwrap().parse::<u16>().unwrap());
                        }
                        "infinite" => {
                            move_time_in_ms = Some(MAX_TIME);
                        }
                        _ => unimplemented!(),
                    }
                }
                let board = &mut Board::from_fen(&fen.unwrap());
                fen = None;

                if perft {
                    println!(
                        "Nodes searched: {}",
                        perft_root(board, false, true, depth.unwrap())
                    );
                    continue;
                }

                let mut engine = Engine::new(board);

                for uci_move in &moves {
                    let (from, to) = (&uci_move[0..2], &uci_move[2..4]);
                    let (from, to) = (Square::from_notation(from), Square::from_notation(to));
                    let piece = engine.board().piece_at(from).unwrap();

                    let mut flag = Flag::None;
                    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
                        if from.rank().abs_diff(to.rank()) == 2 {
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
                        && from.file().abs_diff(to.file()) > 1
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
                            "info depth {depth} score cp {evaluation} time {} pv {}",
                            search_start.elapsed().as_millis(),
                            uci::encode_move(best_move.decode())
                        )
                        // TODO: fix crash when depth goes very high
                    },
                    &mut || search_start.elapsed().as_millis() > think_time,
                );
                println!("bestmove {}", uci::encode_move(best_move.decode()))
            }
            "stop" => {}
            "quit" => return,
            _ => unimplemented!(),
        }
    }
}
