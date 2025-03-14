use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{evaluation::eval_data::EvalNumber, timer::Time};

use super::{IMMEDIATE_CHECKMATE_SCORE, Ply, Search};

enum Mode<'a> {
    Depth(Ply),
    Time {
        timer: &'a Time,
        hard_time_limit: u64,
        soft_time_limit: u64,
    },
    Infinite,
}

#[cfg(target_arch = "wasm32")]
type Bool = bool;

#[cfg(not(target_arch = "wasm32"))]
type Bool = Arc<AtomicBool>;

pub struct TimeManager<'a> {
    mode: Mode<'a>,
    stopped: Bool,
    pondering: Bool,
    mated_in: Option<Ply>,
}

impl<'a> TimeManager<'a> {
    #[must_use]
    pub fn time_limited(
        stopped: Bool,
        pondering: Bool,
        mated_in: Option<Ply>,
        timer: &'a Time,
        hard_time_limit: u64,
        soft_time_limit: u64,
    ) -> Self {
        assert!(hard_time_limit >= soft_time_limit);
        Self {
            stopped,
            pondering,
            mated_in,
            mode: Mode::Time {
                timer,
                hard_time_limit,
                soft_time_limit,
            },
        }
    }
    #[must_use]
    pub const fn depth_limited(
        stopped: Bool,
        pondering: Bool,
        mated_in: Option<Ply>,
        depth: Ply,
    ) -> Self {
        Self {
            stopped,
            pondering,
            mated_in,
            mode: Mode::Depth(depth),
        }
    }
    #[must_use]
    pub const fn infinite(stopped: Bool, pondering: Bool, mated_in: Option<Ply>) -> Self {
        Self {
            stopped,
            pondering,
            mated_in,
            mode: Mode::Infinite,
        }
    }
    #[must_use]
    pub fn hard_stop_inner_search(&self) -> bool {
        if self.is_stopped() {
            return true;
        }
        if self.is_pondering() {
            return false;
        }
        match self.mode {
            Mode::Time {
                timer,
                hard_time_limit,
                ..
            } => timer.milliseconds() > hard_time_limit,
            Mode::Infinite => false,
            Mode::Depth(_) => false,
        }
    }

    #[must_use]
    pub fn hard_stop_iterative_deepening(&self, depth: Ply) -> bool {
        if self.is_stopped() {
            return true;
        }
        if self.is_pondering() {
            return false;
        }

        match self.mode {
            Mode::Time {
                timer,
                hard_time_limit,
                ..
            } => timer.milliseconds() > hard_time_limit,
            Mode::Infinite => false,
            Mode::Depth(max_depth) => depth > max_depth,
        }
    }

    pub fn is_pondering(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        return self.pondering;

        #[cfg(not(target_arch = "wasm32"))]
        return self.pondering.load(Ordering::SeqCst);
    }

    pub fn is_stopped(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        return self.stopped;

        #[cfg(not(target_arch = "wasm32"))]
        return self.stopped.load(Ordering::SeqCst);
    }

    #[must_use]
    pub fn soft_stop(&self, best_score: EvalNumber, best_move_stability: Ply) -> bool {
        if self.is_stopped() {
            return true;
        }
        if self.is_pondering() {
            return false;
        }

        if let Some(ply) = self.mated_in {
            if Search::score_is_checkmate(best_score) {
                if EvalNumber::from(ply) == IMMEDIATE_CHECKMATE_SCORE - best_score.abs() {
                    return true;
                }
            }
        }

        match self.mode {
            Mode::Time {
                timer,
                soft_time_limit,
                hard_time_limit,
            } => {
                const BEST_MOVE_STABILITY_MULTIPLIERS: [u64; 8] =
                    [150, 130, 120, 110, 100, 95, 90, 85];
                let multiplier = BEST_MOVE_STABILITY_MULTIPLIERS[best_move_stability
                    .min(BEST_MOVE_STABILITY_MULTIPLIERS.len() as u8 - 1)
                    as usize];
                let adjusted_time = (soft_time_limit * multiplier) / 100;
                timer.milliseconds() > adjusted_time.min(hard_time_limit)
            }
            Mode::Infinite => false,
            Mode::Depth(_) => false,
        }
    }
}
