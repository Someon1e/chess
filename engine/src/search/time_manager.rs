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
}

pub struct TimeManager<'a> {
    mode: Mode<'a>,
    stopped: Arc<AtomicBool>,
}

impl<'a> TimeManager<'a> {
    #[must_use]
    pub fn time_limited(
        stopped: Arc<AtomicBool>,
        timer: &'a Time,
        hard_time_limit: u64,
        soft_time_limit: u64,
    ) -> Self {
        assert!(hard_time_limit >= soft_time_limit);
        Self {
            mode: Mode::Time {
                timer,
                hard_time_limit,
                soft_time_limit,
            },
            stopped,
        }
    }
    #[must_use]
    pub const fn depth_limited(stopped: Arc<AtomicBool>, depth: Ply) -> Self {
        Self {
            stopped,
            mode: Mode::Depth(depth),
        }
    }
    #[must_use]
    pub fn hard_stop_inner_search(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
            || match self.mode {
                Mode::Time {
                    timer,
                    hard_time_limit,
                    ..
                } => timer.milliseconds() > hard_time_limit,
                Mode::Depth(_) => false,
            }
    }
    #[must_use]
    pub fn hard_stop_iterative_deepening(&self, depth: Ply) -> bool {
        self.stopped.load(Ordering::SeqCst)
            || match self.mode {
                Mode::Time {
                    timer,
                    hard_time_limit,
                    ..
                } => timer.milliseconds() > hard_time_limit,
                Mode::Depth(max_depth) => depth > max_depth,
            }
    }
    #[must_use]
    pub fn soft_stop(&self, best_move_stability: Ply) -> bool {
        self.stopped.load(Ordering::SeqCst)
            || match self.mode {
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
                Mode::Depth(_) => false,
            }
    }
}
