use core::ops::{Range, RangeInclusive};
use core::str::SplitWhitespace;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};

mod go_params;
mod move_encoding;

use go_params::{SearchTime, SearchType};
pub use move_encoding::{decode_move, encode_move};

use crate::evaluation::eval_data::EvalNumber;
use crate::search::pv::Pv;
use crate::{
    board::{Board, square::Square},
    move_generator::move_data::Flag,
    perft::perft_root,
    search::{
        DepthSearchInfo, IMMEDIATE_CHECKMATE_SCORE, Search, TimeManager,
        search_params::{DEFAULT_TUNABLES, Tunable},
        transposition::megabytes_to_capacity,
    },
    timer::Time,
};

pub use self::go_params::GoParameters;

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

    /// Called with UCI output.
    pub out: fn(&str),

    /// Range and default size of the transposition table in megabytes.
    pub hash_option: SpinU16,

    /// Maximum entry count of the transposition table.
    pub transposition_capacity: usize,

    pub stopped: Arc<AtomicBool>,

    pub search_controller: Option<SearchController>,

    #[cfg(feature = "spsa")]
    pub tunables: Tunable,
}

fn output_search(info: DepthSearchInfo, time: u64) {
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
    let pv_string = pv
        .best_line()
        .map(|encoded_move| " ".to_owned() + &encode_move(encoded_move.decode()))
        .collect::<String>();

    let nodes_per_second = if time == 0 {
        69420
    } else {
        (u64::from(nodes) * 1000) / time
    };

    println!(
        "{}",
        &format!(
            "info depth {depth} seldepth {highest_depth} {evaluation_info} time {time} nodes {nodes} nps {nodes_per_second} pv{pv_string}"
        )
    );
}

enum SearchCommand {
    SetPosition((Board, Vec<(Square, Square, Flag)>)),
    Search((Arc<AtomicBool>, u64, u64)),
    SetTranspositionCapacity(usize),
    ClearCacheForNewGame,
}
struct SearchController(Sender<SearchCommand>);
impl SearchController {
    fn new(transposition_capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<SearchCommand>();
        thread::spawn(move || {
            let mut cached_search: Option<Search> = None;
            let mut transposition_capacity = transposition_capacity;
            let mut board = None;
            let mut moves = None;

            for command in receiver {
                match command {
                    SearchCommand::SetTranspositionCapacity(capacity) => {
                        transposition_capacity = capacity;
                        if let Some(search) = &mut cached_search {
                            search.resize_transposition_table(transposition_capacity)
                        }
                    }
                    SearchCommand::SetPosition((new_board, new_moves)) => {
                        board = Some(new_board);
                        moves = Some(new_moves);
                    }
                    SearchCommand::ClearCacheForNewGame => {
                        if let Some(search) = &mut cached_search {
                            search.clear_cache_for_new_game()
                        }
                    }
                    SearchCommand::Search((stopped, soft_time_limit, hard_time_limit)) => {
                        let search_start = Time::now();
                        let time_manager = TimeManager::time_limited(
                            stopped,
                            &search_start,
                            hard_time_limit,
                            soft_time_limit,
                        );
                        let search = if cached_search.is_none() {
                            // First time making search
                            let search = Search::new(
                                board.unwrap(),
                                transposition_capacity,
                                #[cfg(feature = "spsa")]
                                self.tunables,
                            );
                            cached_search = Some(search);
                            cached_search.as_mut().unwrap()
                        } else {
                            // Using cached search
                            let search = cached_search.as_mut().unwrap();
                            search.new_board(board.unwrap());
                            search.clear_for_new_search();
                            search
                        };
                        for (from, to, promotion) in &moves.unwrap() {
                            search.make_move_repetition::<false>(&decode_move(
                                &search.board(),
                                *from,
                                *to,
                                *promotion,
                            ));
                        }
                        moves = None;
                        board = None;

                        let (depth, evaluation) = search.iterative_deepening(
                            &time_manager,
                            &mut |depth_info: DepthSearchInfo| {
                                output_search(depth_info, search_start.milliseconds());
                            },
                        );
                        output_search(
                            DepthSearchInfo {
                                depth,
                                best: (search.pv, evaluation),
                                highest_depth: search.highest_depth,
                                quiescence_call_count: search.quiescence_call_count(),
                            },
                            search_start.milliseconds(),
                        );
                        println!(
                            "{}",
                            &format!(
                                "bestmove {}",
                                encode_move(search.pv.root_best_move().decode())
                            )
                        );
                    }
                }
            }
        });
        Self(sender)
    }
    fn search(&self, stopped: Arc<AtomicBool>, soft_time_limit: u64, hard_time_limit: u64) {
        self.0.send(SearchCommand::Search((
            stopped,
            soft_time_limit,
            hard_time_limit,
        )));
    }
    fn set_position(&self, board: Board, moves: Vec<(Square, Square, Flag)>) {
        self.0.send(SearchCommand::SetPosition((board, moves)));
    }
    fn set_transposition_capacity(&self, transposition_capacity: usize) {
        self.0.send(SearchCommand::SetTranspositionCapacity(
            transposition_capacity,
        ));
    }
    fn clear_cache_for_new_game(&self) {
        self.0.send(SearchCommand::ClearCacheForNewGame);
    }
}

#[cfg(feature = "spsa")]
struct TunableRange {
    pub history_decay: RangeInclusive<i16>,

    pub iir_min_depth: RangeInclusive<u8>,
    pub iir_depth_reduction: RangeInclusive<u8>,

    pub futility_margin: RangeInclusive<i32>,

    pub static_null_margin: RangeInclusive<i32>,
    pub static_null_min_depth: RangeInclusive<u8>,

    pub lmr_min_index: RangeInclusive<usize>,
    pub lmr_min_depth: RangeInclusive<u8>,
    pub lmr_ply_divisor: RangeInclusive<u8>,
    pub lmr_index_divisor: RangeInclusive<u8>,

    pub lmp_base: RangeInclusive<u32>,

    pub nmp_min_depth: RangeInclusive<u8>,
    pub nmp_base_reduction: RangeInclusive<u8>,
    pub nmp_ply_divisor: RangeInclusive<u8>,

    pub aspiration_window_start: RangeInclusive<i32>,
    pub aspiration_window_growth: RangeInclusive<i32>,
}

#[cfg(feature = "spsa")]
const TUNABLE_RANGES: TunableRange = TunableRange {
    history_decay: 2..=20,
    iir_min_depth: 1..=6,
    iir_depth_reduction: 0..=3,
    futility_margin: 60..=165,
    static_null_margin: 30..=90,
    static_null_min_depth: 2..=9,
    lmr_min_index: 2..=6,
    lmr_min_depth: 1..=5,
    lmr_ply_divisor: 6..=16,
    lmr_index_divisor: 6..=13,
    lmp_base: 2..=5,
    nmp_min_depth: 1..=5,
    nmp_base_reduction: 1..=6,
    nmp_ply_divisor: 4..=9,
    aspiration_window_start: 20..=60,
    aspiration_window_growth: 25..=95,
};

impl UCIProcessor {
    pub fn new(max_thinking_time: u64, out: fn(&str), hash_option: SpinU16) -> Self {
        let megabytes = hash_option.default as usize;
        let transposition_capacity = megabytes_to_capacity(megabytes);

        Self {
            max_thinking_time,
            fen: None,
            moves: Vec::new(),
            out,
            stopped: Arc::new(AtomicBool::new(false)),
            hash_option,
            transposition_capacity,
            search_controller: None,
            #[cfg(feature = "spsa")]
            tunables: DEFAULT_TUNABLES,
        }
    }
    fn set_transposition_capacity(&mut self, transposition_capacity: usize) {
        self.transposition_capacity = transposition_capacity;
        if let Some(search_controller) = &mut self.search_controller {
            search_controller.set_transposition_capacity(transposition_capacity);
        }
    }
}

impl UCIProcessor {
    /// Outputs `id` command, `option` commands, and `uciok`
    pub fn uci(&self) {
        let min_hash = self.hash_option.range.start;
        let default_hash = self.hash_option.default;
        let max_hash = self.hash_option.range.end - 1;
        let mut options = format!(
            "option name Hash type spin default {default_hash} min {min_hash} max {max_hash}
option name Threads type spin default 1 min 1 max 1"
        );

        #[cfg(feature = "spsa")]
        {
            macro_rules! spin {
                ($name:expr, $default:expr, $min:expr, $max:expr) => {
                    options.push_str(&format!(
                        "\noption name {} type spin default {} min {} max {}",
                        $name, $default, $min, $max
                    ));
                };
            }
            macro_rules! define_spins {
                ($($field:ident),*) => {
                    $(
                        spin!(
                            stringify!($field),
                            DEFAULT_TUNABLES.$field,
                            TUNABLE_RANGES.$field.start(),
                            TUNABLE_RANGES.$field.end()
                        );
                    )*
                };
            }

            define_spins!(
                history_decay,
                iir_min_depth,
                iir_depth_reduction,
                futility_margin,
                static_null_margin,
                static_null_min_depth,
                lmr_min_index,
                lmr_min_depth,
                lmr_ply_divisor,
                lmr_index_divisor,
                lmp_base,
                nmp_min_depth,
                nmp_base_reduction,
                nmp_ply_divisor,
                aspiration_window_start,
                aspiration_window_growth
            );
        }

        (self.out)(&format!(
            "id name {} {}
id author someone
{}
uciok",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            options
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

        macro_rules! handle_option {
            ($option_name:expr, $value:expr, $self:expr, { $($field:ident),* $(,)? }) => {
                match $option_name {
                    $(
                        #[cfg(feature = "spsa")]
                        stringify!($field) => {
                            let parsed_value = $value.expect("Missing value").parse().unwrap();
                            assert!(TUNABLE_RANGES.$field.contains(&parsed_value));
                            $self.tunables.$field = parsed_value;
                        }
                    )*
                    _ => {
                        if $value.is_none() {
                            panic!("Unknown option name (or missing value label)");
                        } else {
                            panic!("Unknown option name");
                        }
                    }
                }
            };
        }

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

            option_name => handle_option!(
                option_name,
                value,
                self,
                {
                    history_decay,
                    iir_min_depth,
                    iir_depth_reduction,
                    futility_margin,
                    static_null_margin,
                    static_null_min_depth,
                    lmr_min_index,
                    lmr_min_depth,
                    lmr_ply_divisor,
                    lmr_index_divisor,
                    lmp_base,
                    nmp_min_depth,
                    nmp_base_reduction,
                    nmp_ply_divisor,
                    aspiration_window_start,
                    aspiration_window_growth
                }
            ),
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
                        let (from, to) = (
                            Square::from_notation(from).unwrap(),
                            Square::from_notation(to).unwrap(),
                        );
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
    pub fn go(&mut self, parameters: GoParameters) {
        let mut board = Board::from_fen(self.fen.as_ref().unwrap()).unwrap();

        if matches!(parameters.search_type(), SearchType::Perft) {
            for (from, to, promotion) in &self.moves {
                board.make_move(&decode_move(&board, *from, *to, *promotion));
            }

            let search_start = Time::now();
            let nodes = perft_root(&mut board, parameters.depth().unwrap(), self.out);
            let time = search_start.milliseconds();
            let nodes_per_second = if time == 0 { 0 } else { (nodes * 1000) / time };
            (self.out)(&format!(
                "Searched {nodes} nodes in {time} milliseconds ({nodes_per_second} nodes per second)",
            ));
            return;
        }

        let (hard_time_limit, soft_time_limit) = match parameters.move_time().unwrap() {
            SearchTime::Infinite => (self.max_thinking_time, self.max_thinking_time),
            SearchTime::Fixed(move_time) => (move_time, move_time),
            SearchTime::Info(info) => {
                let clock_time = (if board.white_to_move {
                    info.white_time
                } else {
                    info.black_time
                })
                .unwrap();

                let increment = (if board.white_to_move {
                    info.white_increment
                } else {
                    info.black_increment
                })
                .map_or_else(|| 0, core::num::NonZero::get);

                Search::calculate_time(clock_time, increment, self.max_thinking_time)
            }
        };

        let search_controller = if self.search_controller.is_some() {
            self.search_controller.as_ref()
        } else {
            self.search_controller = Some(SearchController::new(self.transposition_capacity));
            self.search_controller.as_ref()
        }
        .unwrap();
        search_controller.set_position(board, self.moves.clone());
        search_controller.search(self.stopped.clone(), soft_time_limit, hard_time_limit);
    }

    /// Stop calculating as soon as possible.
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    /// This is sent to the engine when the next search (started with "position" and "go") will be from
    /// a different game. This can be a new game the engine should play or a new game it should analyse but
    /// also the next position from a testsuite with positions only.
    pub fn ucinewgame(&mut self) {
        // New game, so old data like the transposition table will not help
        if let Some(search_controller) = &mut self.search_controller {
            search_controller.clear_cache_for_new_game();
        }
    }
}
