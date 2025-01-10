use core::str::SplitWhitespace;
use std::ops::Range;

mod go_params;

use go_params::{SearchTime, SearchType};

use crate::{
    board::{piece::Piece, square::Square, Board},
    move_generator::move_data::{Flag, Move},
    perft::perft_root,
    search::{
        encoded_move::EncodedMove, transposition::megabytes_to_capacity, DepthSearchInfo, Search,
        TimeManager, IMMEDIATE_CHECKMATE_SCORE,
    },
    timer::Time,
};

use self::go_params::GoParameters;

/// Encodes a move in uci notation.
#[must_use]
pub fn encode_move(move_data: Move) -> String {
    const NULL_MOVE: Move = EncodedMove::NONE.decode();
    if move_data == NULL_MOVE {
        return "0000".to_owned();
    }

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

/// # Panics
///
/// Will panic if there is no friendly piece at `from`.
#[must_use]
pub fn decode_move(board: &Board, from: Square, to: Square, promotion: Flag) -> Move {
    let piece = board
        .friendly_piece_at(from)
        .expect("Tried to play illegal move");

    let mut flag = promotion;
    if piece == Piece::WhitePawn || piece == Piece::BlackPawn {
        if from.rank().abs_diff(to.rank()) == 2 {
            flag = Flag::PawnTwoUp;
        } else if board.game_state.en_passant_square == Some(to) {
            flag = Flag::EnPassant;
        }
    } else if (piece == Piece::BlackKing || piece == Piece::WhiteKing)
        && from.file().abs_diff(to.file()) > 1
    {
        flag = Flag::Castle;
    }

    Move { from, to, flag }
}

/// An value within a range.
pub struct SpinU16 {
    range: Range<u16>,
    default: u16,
}
impl SpinU16 {
    #[must_use]
    pub fn new(range: Range<u16>, default: u16) -> Self {
        assert!(range.contains(&default));

        Self { range, default }
    }
}

/// Handles UCI input and output.
pub struct UCIProcessor {
    /// Maximum time to search, in milliseconds.
    pub max_thinking_time: u64,

    /// FEN to be used.
    pub fen: Option<String>,

    /// Moves to be played after FEN.
    pub moves: Vec<(Square, Square, Flag)>,

    /// Search instance.
    pub search: Option<Search>,

    /// Called with UCI output.
    pub out: fn(&str),

    /// Range and default size of the transposition table in megabytes.
    pub hash_option: SpinU16,

    /// Maximum entry count of the transposition table.
    pub transposition_capacity: usize,
}

impl UCIProcessor {
    pub fn new(max_thinking_time: u64, out: fn(&str), hash_option: SpinU16) -> Self {
        let megabytes = hash_option.default as usize;
        let transposition_capacity = megabytes_to_capacity(megabytes);

        Self {
            max_thinking_time,
            fen: None,
            moves: Vec::new(),
            search: None,
            out,
            hash_option,
            transposition_capacity,
        }
    }
    fn set_transposition_capacity(&mut self, transposition_capacity: usize) {
        self.transposition_capacity = transposition_capacity;
        if let Some(search) = &mut self.search {
            search.resize_transposition_table(transposition_capacity);
        }
    }
}

impl UCIProcessor {
    /// Outputs `id` command, `option` commands, and `uciok`
    pub fn uci(&self) {
        let min_hash = self.hash_option.range.start;
        let default_hash = self.hash_option.default;
        let max_hash = self.hash_option.range.end - 1;

        (self.out)(&format!(
            "id name {} {}
id author someone
option name Hash type spin default {default_hash} min {min_hash} max {max_hash}
option name Threads type spin default 1 min 1 max 1
uciok",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ));
    }

    /// This should output `readyok`.
    pub fn isready(&self) {
        (self.out)("readyok");
    }

    pub fn setoption(&mut self, args: &str) {
        let trimmed = args.trim();

        let name_index = trimmed.find("name ").expect("Did not find name");
        let value_index = trimmed.find(" value ");

        let (name, value) = if let Some(value_index) = value_index {
            let name = &trimmed[(name_index + 5)..value_index];
            let value = &trimmed[(value_index + 7)..];
            (name, Some(value))
        } else {
            let name = &trimmed[(name_index + 5)..];
            (name, None)
        };

        match name.trim().to_lowercase().as_str() {
            "hash" => {
                let megabytes = value.expect("Missing value").parse().unwrap();
                assert!(self.hash_option.range.contains(&megabytes));

                self.set_transposition_capacity(megabytes_to_capacity(megabytes.into()));
            }
            "threads" => {
                let threads: u16 = value.expect("Missing value").parse().unwrap();
                assert!(threads == 1, "Only supports single thread");
            }
            _ => {
                if value.is_none() {
                    panic!("Unknown option name (or missing value label)")
                } else {
                    panic!("Unknown option name")
                }
            }
        }
    }

    /// # Panics
    ///
    /// Will panic if there are invalid moves.
    pub fn position(&mut self, args: &mut SplitWhitespace) {
        self.moves.clear();

        let mut startpos = true;
        let mut building_fen = String::new();

        while let Some(label) = args.next() {
            match label {
                "startpos" => startpos = true,
                "fen" => startpos = false,
                "moves" => {
                    for uci_move in args.by_ref() {
                        let (from, to) = (&uci_move[0..2], &uci_move[2..4]);
                        let (from, to) = (Square::from_notation(from), Square::from_notation(to));
                        let promotion = match uci_move.chars().nth(4) {
                            None => Flag::None,
                            Some('q') => Flag::QueenPromotion,
                            Some('r') => Flag::RookPromotion,
                            Some('n') => Flag::KnightPromotion,
                            Some('b') => Flag::BishopPromotion,
                            _ => {
                                panic!("Invalid promotion notation in {uci_move}")
                            }
                        };
                        self.moves.push((from, to, promotion));
                    }
                }
                _ => {
                    if !startpos {
                        building_fen.push_str(label);
                        building_fen.push(' ');
                    }
                }
            }
        }

        self.fen = Some(if startpos {
            Board::START_POSITION_FEN.to_owned()
        } else {
            building_fen
        });
    }

    /// # Panics
    ///
    /// Will panic if there are missing parameters.
    pub fn go(&mut self, args: &mut SplitWhitespace) {
        let mut parameters = GoParameters::empty();
        parameters.parse(args);

        let mut board = Board::from_fen(self.fen.as_ref().unwrap());

        if matches!(parameters.search_type, SearchType::Perft) {
            for (from, to, promotion) in &self.moves {
                board.make_move(&decode_move(&board, *from, *to, *promotion));
            }

            let search_start = Time::now();
            let nodes = perft_root(&mut board, parameters.depth.unwrap(), self.out);
            let time = search_start.milliseconds();
            let nodes_per_second = if time == 0 { 0 } else { (nodes * 1000) / time };
            (self.out)(&format!(
                "Searched {nodes} nodes in {time} milliseconds ({nodes_per_second} nodes per second)",
            ));
            return;
        }

        let search = if self.search.is_none() {
            // First time making search
            let search = Search::new(board, self.transposition_capacity);
            self.search = Some(search);
            self.search.as_mut().unwrap()
        } else {
            // Using cached search
            let search = self.search.as_mut().unwrap();
            search.new_board(board);
            search.clear_for_new_search();
            search
        };
        for (from, to, promotion) in &self.moves {
            search.make_move(&decode_move(search.board(), *from, *to, *promotion));
        }

        let (hard_time_limit, soft_time_limit) = match parameters.move_time.unwrap() {
            SearchTime::Infinite => (self.max_thinking_time, self.max_thinking_time),
            SearchTime::Fixed(move_time) => (move_time, move_time),
            SearchTime::Info(info) => {
                let clock_time = (if search.board().white_to_move {
                    info.white_time
                } else {
                    info.black_time
                })
                .unwrap();

                let increment = (if search.board().white_to_move {
                    info.white_increment
                } else {
                    info.black_increment
                })
                .map_or_else(|| 0, core::num::NonZero::get);

                let max_time = self.max_thinking_time.min(clock_time / 2);
                let hard_time_limit = (clock_time / 6 + increment * 2).min(max_time);
                let soft_time_limit = (clock_time / 24 + increment / 2).min(hard_time_limit);

                (hard_time_limit, soft_time_limit)
            }
        };

        let search_start = Time::now();
        let output_info = |info: DepthSearchInfo| {
            let (pv, evaluation) = info.best;
            let depth = info.depth;
            let highest_depth = info.highest_depth;
            let nodes = info.quiescence_call_count;

            let evaluation_info = if Search::score_is_checkmate(evaluation) {
                format!(
                    "score mate {}",
                    (evaluation - IMMEDIATE_CHECKMATE_SCORE).abs() * evaluation.signum()
                )
            } else {
                format!("score cp {evaluation}")
            };
            let time = search_start.milliseconds();
            let pv_string = pv.pv_table[0]
                .iter()
                .take(pv.pv_length[0] as usize)
                .map(|encoded_move| " ".to_owned() + &encode_move(encoded_move.decode()))
                .collect::<String>();

            let nodes_per_second = if time == 0 {
                69420
            } else {
                (u64::from(nodes) * 1000) / time
            };

            (self.out)(&format!(
                "info depth {depth} seldepth {highest_depth} {evaluation_info} time {time} nodes {nodes} nps {nodes_per_second} pv{pv_string}"
            ));
        };

        let time_manager =
            TimeManager::time_limited(&search_start, hard_time_limit, soft_time_limit);
        let (depth, evaluation) = search.iterative_deepening(&time_manager, &mut |depth_info| {
            output_info(depth_info);
        });

        output_info(DepthSearchInfo {
            depth,
            best: (&search.pv, evaluation),
            highest_depth: search.highest_depth,
            quiescence_call_count: search.quiescence_call_count(),
        });
        (self.out)(&format!(
            "bestmove {}",
            encode_move(search.pv.root_best_move().decode())
        ));
    }

    /// Stop calculating as soon as possible.
    pub fn stop(&self) {
        todo!("Stop search immediately")
    }

    /// This is sent to the engine when the next search (started with "position" and "go") will be from
    /// a different game. This can be a new game the engine should play or a new game it should analyse but
    /// also the next position from a testsuite with positions only.
    pub fn ucinewgame(&mut self) {
        // New game, so old data like the transposition table will not help
        if let Some(search) = &mut self.search {
            search.clear_cache_for_new_game();
        }
    }
}
