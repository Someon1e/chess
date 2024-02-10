use std::{
    io::{stdin, BufRead},
    str::SplitWhitespace,
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

#[derive(Default)]
struct GoParameters {
    white_time: Option<u128>,
    black_time: Option<u128>,
    white_increment: Option<u128>,
    black_increment: Option<u128>,
    moves_to_go: Option<u64>,
    perft: bool,
    depth: Option<u16>,
    nodes: Option<u64>,
    find_mate: Option<u64>,
    move_time_in_ms: Option<u128>,
}
impl GoParameters {
    fn empty() -> Self {
        Self {
            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,
            perft: false,
            depth: None,
            nodes: None,
            find_mate: None,
            move_time_in_ms: None,
        }
    }
    fn parse(&mut self, args: &mut SplitWhitespace) {
        while let Some(label) = args.next() {
            match label {
                "searchmoves" => {}
                "ponder" => {}
                "wtime" => {
                    self.white_time = Some(args.next().unwrap().parse::<u128>().unwrap());
                }
                "btime" => {
                    self.black_time = Some(args.next().unwrap().parse::<u128>().unwrap());
                }
                "winc" => {
                    self.white_increment = Some(args.next().unwrap().parse::<u128>().unwrap());
                }
                "binc" => {
                    self.black_increment = Some(args.next().unwrap().parse::<u128>().unwrap());
                }
                "movestogo" => {
                    self.moves_to_go = Some(args.next().unwrap().parse::<u64>().unwrap());
                }
                "depth" => {
                    self.depth = Some(args.next().unwrap().parse::<u16>().unwrap());
                }
                "nodes" => {
                    self.nodes = Some(args.next().unwrap().parse::<u64>().unwrap());
                }
                "mate" => {
                    self.find_mate = Some(args.next().unwrap().parse::<u64>().unwrap());
                }
                "movetime" => {
                    self.move_time_in_ms = Some(args.next().unwrap().parse::<u128>().unwrap());
                }
                "perft" => {
                    self.perft = true;
                    self.depth = Some(args.next().unwrap().parse::<u16>().unwrap());
                }
                "infinite" => {
                    self.move_time_in_ms = Some(MAX_TIME);
                }
                _ => unimplemented!(),
            }
        }
    }
}

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
                let mut parameters = GoParameters::empty();
                parameters.parse(&mut args);

                let board = &mut Board::from_fen(&fen.unwrap());
                fen = None;

                if parameters.perft {
                    println!(
                        "Nodes searched: {}",
                        perft_root(board, false, true, parameters.depth.unwrap())
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

                let think_time = parameters.move_time_in_ms.unwrap_or_else(|| {
                    let clock_time = (if engine.board().white_to_move {
                        parameters.white_time
                    } else {
                        parameters.black_time
                    })
                    .unwrap();
                    let increment = (if engine.board().white_to_move {
                        parameters.white_increment
                    } else {
                        parameters.black_increment
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
