use std::fmt::Write;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::board::Board;
use crate::board::square::Square;
use crate::move_generator::move_data::Flag;
use crate::search::encoded_move::EncodedMove;
use crate::search::pv::Pv;
use crate::search::{DepthSearchInfo, IMMEDIATE_CHECKMATE_SCORE, Search, TimeManager};
use crate::timer::Time;
use crate::uci::encode_move;

#[cfg(target_arch = "wasm32")]
type Bool = bool;

#[cfg(not(target_arch = "wasm32"))]
type Bool = Arc<AtomicBool>;

use super::go_params::SearchTime;
use super::{PonderInfo, decode_move};

fn output_search(out: fn(&str), info: &DepthSearchInfo, time: u64) {
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

    out(&format!(
        "info depth {depth} seldepth {highest_depth} {evaluation_info} time {time} nodes {nodes} nps {nodes_per_second} pv{pv_string}"
    ));
}

fn search(
    out: fn(&str),
    cached_search: &mut Option<Search>,
    board: &mut Option<Board>,
    moves: &mut Option<Vec<(Square, Square, Flag)>>,
    transposition_capacity: usize,
    search_time: SearchTime,
    stopped: Bool,
    ponder_info: PonderInfo,
) {
    let search_start = Time::now();

    let search = if cached_search.is_none() {
        // First time making search
        let search = Search::new(
            board.take().unwrap(),
            transposition_capacity,
            #[cfg(feature = "spsa")]
            self.tunables,
        );
        *cached_search = Some(search);
        cached_search.as_mut().unwrap()
    } else {
        // Using cached search
        let search = cached_search.as_mut().unwrap();
        search.new_board(board.take().unwrap());
        search.clear_for_new_search();
        search
    };
    for (from, to, promotion) in &moves.take().unwrap() {
        search.make_move_repetition::<false>(&decode_move(search.board(), *from, *to, *promotion));
    }

    let time_manager = match search_time {
        SearchTime::Infinite => TimeManager::infinite(stopped, ponder_info.is_pondering),
        SearchTime::Fixed(move_time) => TimeManager::time_limited(
            stopped,
            ponder_info.is_pondering,
            &search_start,
            move_time,
            move_time,
        ),
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

            let (hard_time_limit, soft_time_limit) = Search::calculate_time(clock_time, increment);
            TimeManager::time_limited(
                stopped,
                ponder_info.is_pondering,
                &search_start,
                hard_time_limit,
                soft_time_limit,
            )
        }
        _ => panic!("Unknown time control"),
    };

    let (mut root_best_move, mut root_best_reply) = (EncodedMove::NONE, EncodedMove::NONE);
    let mut try_update = |pv: &Pv| {
        let new_best_move = pv.root_best_move();
        if new_best_move != root_best_move {
            root_best_reply = EncodedMove::NONE;
            root_best_move = new_best_move;
        }

        let new_best_reply = pv.root_best_reply();
        if !new_best_reply.is_none() {
            root_best_reply = new_best_reply;
        }
    };

    let (depth, evaluation) =
        search.iterative_deepening(&time_manager, &mut |depth_info: DepthSearchInfo| {
            try_update(&depth_info.best.0);
            output_search(out, &depth_info, search_start.milliseconds());
        });

    try_update(&search.pv);
    output_search(
        out,
        &DepthSearchInfo {
            depth,
            best: (search.pv, evaluation),
            highest_depth: search.highest_depth,
            quiescence_call_count: search.quiescence_call_count(),
        },
        search_start.milliseconds(),
    );

    let mut output = format!("bestmove {}", encode_move(root_best_move.decode()),);
    if !root_best_reply.is_none() {
        write!(output, " ponder {}", encode_move(root_best_reply.decode())).unwrap();
    }

    out(&output);
}

#[cfg(not(target_arch = "wasm32"))]
mod search_controller {
    use std::sync::atomic::AtomicBool;
    use std::sync::mpsc::{self, Sender};

    use std::sync::Arc;
    use std::thread;

    use crate::board::Board;
    use crate::board::square::Square;
    use crate::move_generator::move_data::Flag;
    use crate::search::Search;
    use crate::uci::go_params::SearchTime;
    use crate::uci::{Bool, PonderInfo};

    use super::search;

    enum SearchCommand {
        SetPosition((Board, Vec<(Square, Square, Flag)>)),
        Search((Bool, SearchTime, PonderInfo)),
        SetTranspositionCapacity(usize),
        ClearCacheForNewGame,
    }

    pub struct SearchController(Sender<SearchCommand>);
    impl SearchController {
        pub fn new(out: fn(&str), transposition_capacity: usize) -> Self {
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
                                search.resize_transposition_table(transposition_capacity);
                            }
                        }
                        SearchCommand::SetPosition((new_board, new_moves)) => {
                            board = Some(new_board);
                            moves = Some(new_moves);
                        }
                        SearchCommand::ClearCacheForNewGame => {
                            if let Some(search) = &mut cached_search {
                                search.clear_cache_for_new_game();
                            }
                        }
                        SearchCommand::Search((stopped, search_time, ponder_info)) => search(
                            out,
                            &mut cached_search,
                            &mut board,
                            &mut moves,
                            transposition_capacity,
                            search_time,
                            stopped,
                            ponder_info,
                        ),
                    }
                }
            });
            Self(sender)
        }
        pub fn search(
            &self,
            stopped: Arc<AtomicBool>,
            search_time: SearchTime,
            ponder_info: PonderInfo,
        ) {
            self.0
                .send(SearchCommand::Search((stopped, search_time, ponder_info)))
                .unwrap();
        }
        pub fn set_position(&self, board: Board, moves: Vec<(Square, Square, Flag)>) {
            self.0
                .send(SearchCommand::SetPosition((board, moves)))
                .unwrap();
        }
        pub fn set_transposition_capacity(&self, transposition_capacity: usize) {
            self.0
                .send(SearchCommand::SetTranspositionCapacity(
                    transposition_capacity,
                ))
                .unwrap();
        }
        pub fn clear_cache_for_new_game(&self) {
            self.0.send(SearchCommand::ClearCacheForNewGame).unwrap();
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod search_controller {
    use crate::board::Board;
    use crate::board::square::Square;
    use crate::move_generator::move_data::Flag;
    use crate::search::Search;
    use crate::uci::PonderInfo;
    use crate::uci::go_params::SearchTime;

    use super::{Bool, search};

    pub struct SearchController {
        out: fn(&str),
        cached_search: Option<Search>,
        board: Option<Board>,
        moves: Option<Vec<(Square, Square, Flag)>>,
        transposition_capacity: usize,
    }
    impl SearchController {
        pub fn new(out: fn(&str), transposition_capacity: usize) -> Self {
            Self {
                out,
                cached_search: None,
                board: None,
                moves: None,
                transposition_capacity,
            }
        }
        pub fn search(&mut self, stopped: Bool, search_time: SearchTime, ponder_info: PonderInfo) {
            search(
                self.out,
                &mut self.cached_search,
                &mut self.board,
                &mut self.moves,
                self.transposition_capacity,
                search_time,
                stopped,
                ponder_info,
            );
        }
        pub fn set_position(&mut self, board: Board, moves: Vec<(Square, Square, Flag)>) {
            self.board = Some(board);
            self.moves = Some(moves);
        }
        pub fn set_transposition_capacity(&mut self, transposition_capacity: usize) {
            self.transposition_capacity = transposition_capacity;
            if let Some(search) = &mut self.cached_search {
                search.resize_transposition_table(transposition_capacity);
            }
        }
        pub fn clear_cache_for_new_game(&mut self) {
            if let Some(search) = &mut self.cached_search {
                search.clear_cache_for_new_game();
            }
        }
    }
}

pub use search_controller::SearchController;
