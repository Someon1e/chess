use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::timer::Time;

use super::Ply;

enum Mode<'a> {
    Depth(Ply),
    Time {
        timer: &'a Time,
        hard_time_limit: u64,
        soft_time_limit: u64,
    },
    Infinite,
}

pub struct TimeManager<'a> {
    mode: Mode<'a>,
    stopped: Arc<AtomicBool>,
    pondering: Arc<AtomicBool>,
}

impl<'a> TimeManager<'a> {
    #[must_use]
    pub fn time_limited(
        stopped: Arc<AtomicBool>,
        pondering: Arc<AtomicBool>,
        timer: &'a Time,
        hard_time_limit: u64,
        soft_time_limit: u64,
    ) -> Self {
        assert!(hard_time_limit >= soft_time_limit);
        Self {
            pondering,
            mode: Mode::Time {
                timer,
                hard_time_limit,
                soft_time_limit,
            },
            stopped,
        }
    }
    #[must_use]
    pub const fn depth_limited(
        stopped: Arc<AtomicBool>,
        pondering: Arc<AtomicBool>,
        depth: Ply,
    ) -> Self {
        Self {
            stopped,
            pondering,
            mode: Mode::Depth(depth),
        }
    }
    #[must_use]
    pub const fn infinite(stopped: Arc<AtomicBool>, pondering: Arc<AtomicBool>) -> Self {
        Self {
            pondering,
            stopped,
            mode: Mode::Infinite,
        }
    }
    #[must_use]
    pub fn hard_stop_inner_search(&self) -> bool {
        if self.stopped.load(Ordering::SeqCst) {
            return true;
        }
        if self.pondering.load(Ordering::SeqCst) {
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
        if self.stopped.load(Ordering::SeqCst) {
            return true;
        }
        if self.pondering.load(Ordering::SeqCst) {
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
    #[must_use]
    pub fn soft_stop(&self, best_move_stability: Ply) -> bool {
        if self.stopped.load(Ordering::SeqCst) {
            return true;
        }
        if self.pondering.load(Ordering::SeqCst) {
            return false;
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
