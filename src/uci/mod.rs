use core::str::SplitWhitespace;

use crate::{
    board::{piece::Piece, square::Square, Board},
    engine::Engine,
    move_generator::move_data::{Flag, Move},
    perft::perft_root,
    timer::timer::Time,
};

pub fn encode_move(move_data: Move) -> String {
    let mut encoded = String::with_capacity(4);
    encoded.push_str(&move_data.from.to_notation());
    encoded.push_str(&move_data.to.to_notation());

    match move_data.flag {
        Flag::QueenPromotion => encoded.push('q'),
        Flag::RookPromotion => encoded.push('r'),
        Flag::KnightPromotion => encoded.push('n'),
        Flag::BishopPromotion => encoded.push('b'),
        _ => {}
    };
    encoded
}
pub fn decode_move(board: &Board, uci_move: &str) -> Move {
    let (from, to) = (&uci_move[0..2], &uci_move[2..4]);
    let (from, to) = (Square::from_notation(from), Square::from_notation(to));
    let piece = board.piece_at(from).unwrap();

    let mut flag = Flag::None;
    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
        if from.rank().abs_diff(to.rank()) == 2 {
            flag = Flag::PawnTwoUp
        } else if board.game_state.en_passant_square == Some(to) {
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

    Move { from, to, flag }
}

#[derive(Default)]
pub struct GoParameters {
    pub white_time: Option<u128>,
    pub black_time: Option<u128>,
    pub white_increment: Option<u128>,
    pub black_increment: Option<u128>,
    pub moves_to_go: Option<u64>,
    pub perft: bool,
    pub depth: Option<u16>,
    pub infinite: bool,
    pub nodes: Option<u64>,
    pub find_mate: Option<u64>,
    pub move_time_in_ms: Option<u128>,
}
impl GoParameters {
    pub fn empty() -> Self {
        Self {
            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,
            perft: false,
            depth: None,
            infinite: false,
            nodes: None,
            find_mate: None,
            move_time_in_ms: None,
        }
    }
    pub fn parse(&mut self, args: &mut SplitWhitespace) {
        while let Some(label) = args.next() {
            match label {
                "searchmoves" => {}
                "ponder" => {}
                "wtime" => {
                    self.white_time = Some(args.next().unwrap().parse().unwrap());
                }
                "btime" => {
                    self.black_time = Some(args.next().unwrap().parse().unwrap());
                }
                "winc" => {
                    self.white_increment = Some(args.next().unwrap().parse().unwrap());
                }
                "binc" => {
                    self.black_increment = Some(args.next().unwrap().parse().unwrap());
                }
                "movestogo" => {
                    self.moves_to_go = Some(args.next().unwrap().parse().unwrap());
                }
                "depth" => {
                    self.depth = Some(args.next().unwrap().parse().unwrap());
                }
                "nodes" => {
                    self.nodes = Some(args.next().unwrap().parse().unwrap());
                }
                "mate" => {
                    self.find_mate = Some(args.next().unwrap().parse().unwrap());
                }
                "movetime" => {
                    self.move_time_in_ms = Some(args.next().unwrap().parse().unwrap());
                }
                "perft" => {
                    self.perft = true;
                    self.depth = Some(args.next().unwrap().parse().unwrap());
                }
                "infinite" => {
                    self.infinite = true;
                }
                _ => unimplemented!(),
            }
        }
    }
}

pub struct UCIProcessor {
    pub max_thinking_time: u128,

    pub moves: Vec<String>,
    pub fen: Option<String>,

    pub out: fn(&str),
}

impl UCIProcessor {
    pub fn uci(&self) {
        (self.out)(
            "id name chess
id author someone
uciok",
        );
    }
    pub fn isready(&self) {
        (self.out)("readyok");
    }
    pub fn position(&mut self, args: &mut SplitWhitespace) {
        let mut startpos = true;
        let mut building_fen = String::new();
        while let Some(label) = args.next() {
            match label {
                "moves" => {
                    self.moves.clear();
                    while let Some(uci_move) = args.next() {
                        self.moves.push(uci_move.to_owned());
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
            self.fen = Some(Board::START_POSITION_FEN.to_owned());
        } else {
            self.fen = Some(building_fen)
        }
    }
    pub fn go(&mut self, args: &mut SplitWhitespace) {
        let mut parameters = GoParameters::empty();
        parameters.parse(args);

        let board = &mut Board::from_fen(self.fen.as_ref().unwrap());
        self.fen = None;

        if parameters.perft {
            (self.out)(&format!(
                "Nodes searched: {}",
                perft_root(board, false, true, parameters.depth.unwrap(), self.out)
            ));
            return;
        }

        let mut engine = Engine::new(board);

        for uci_move in &self.moves {
            engine.make_move(&decode_move(engine.board(), uci_move))
        }
        self.moves.clear();
        (self.out)(&engine.board().to_fen());

        let think_time = if parameters.infinite {
            self.max_thinking_time
        } else {
            parameters.move_time_in_ms.unwrap_or_else(|| {
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
                (clock_time / 20 + increment / 2).min(self.max_thinking_time)
            })
        };

        let search_start = Time::now();
        let (best_move, _evaluation) = engine.iterative_deepening(
            &mut |depth, (best_move, evaluation)| {
                (self.out)(&format!(
                    "info depth {depth} score cp {evaluation} time {} pv {}",
                    search_start.miliseconds(),
                    encode_move(best_move.decode())
                ))
                // TODO: fix crash when depth goes very high
            },
            &mut || search_start.miliseconds() as u128 > think_time,
        );
        (self.out)(&format!("bestmove {}", encode_move(best_move.decode())))
    }

    pub fn stop(&self) {}
    pub fn ucinewgame(&self) {}
}
