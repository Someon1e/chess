use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::{evaluation::eval_data::EvalNumber, timer::Time};

use super::{IMMEDIATE_CHECKMATE_SCORE, Ply, Search};

#[cfg(target_arch = "wasm32")]
type Bool = bool;

#[cfg(not(target_arch = "wasm32"))]
type Bool = Arc<AtomicBool>;

pub struct TimeManager<'a> {
    depth_limit: Option<Ply>,
    node_limit: Option<NodeLimit>,
    real_time: Option<RealTime<'a>>,

    stopped: Bool,
    pondering: Bool,
    mated_in: Option<Ply>,
}

pub struct NodeLimit {
    soft_limit: u64,
    hard_limit: u64,
}
impl NodeLimit {
    pub const fn new(hard_limit: u64, soft_limit: u64) -> Self {
        assert!(hard_limit >= soft_limit);
        Self {
            soft_limit,
            hard_limit,
        }
    }
}

pub struct RealTime<'a> {
    timer: &'a Time,
    hard_time_limit: u64,
    soft_time_limit: u64,
}
impl<'a> RealTime<'a> {
    pub fn new(timer: &'a Time, hard_time_limit: u64, soft_time_limit: u64) -> Self {
        assert!(hard_time_limit >= soft_time_limit);
        Self {
            timer,
            hard_time_limit,
            soft_time_limit,
        }
    }
}

impl<'a> TimeManager<'a> {
    #[must_use]
    pub fn new(
        depth_limit: Option<Ply>,
        node_limit: Option<NodeLimit>,
        real_time: Option<RealTime<'a>>,

        stopped: Bool,
        pondering: Bool,
        mated_in: Option<Ply>,
    ) -> Self {
        Self {
            depth_limit,
            node_limit,
            real_time,

            stopped,
            pondering,
            mated_in,
        }
    }

    #[must_use]
    pub fn time_limited(
        stopped: Bool,
        pondering: Bool,
        mated_in: Option<Ply>,
        real_time: Option<RealTime<'a>>,
    ) -> Self {
        Self {
            stopped,
            pondering,
            mated_in,
            real_time,
            depth_limit: None,
            node_limit: None,
        }
    }

    /// Stop when iterative deepening depth reaches `depth`
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
            depth_limit: Some(depth),
            node_limit: None,
            real_time: None,
        }
    }

    #[must_use]
    pub const fn node_limited(
        stopped: Bool,
        pondering: Bool,
        mated_in: Option<Ply>,
        node_limit: NodeLimit,
    ) -> Self {
        Self {
            stopped,
            pondering,
            mated_in,
            depth_limit: None,
            node_limit: Some(node_limit),
            real_time: None,
        }
    }

    /// Only ends when `stopped` is true
    #[must_use]
    pub const fn infinite(stopped: Bool, pondering: Bool, mated_in: Option<Ply>) -> Self {
        Self {
            stopped,
            pondering,
            mated_in,
            depth_limit: None,
            node_limit: None,
            real_time: None,
        }
    }

    #[must_use]
    pub fn hard_stop_inner_search(&self, node_count: u64) -> bool {
        if self.is_stopped() {
            return true;
        }
        if self.is_pondering() {
            return false;
        }
        if self
            .node_limit
            .as_ref()
            .is_some_and(|node_limit| node_count >= node_limit.hard_limit)
        {
            return true;
        }

        if self
            .real_time
            .as_ref()
            .is_some_and(|real_time| real_time.timer.milliseconds() > real_time.hard_time_limit)
        {
            return true;
        }
        return false;
    }

    #[must_use]
    pub fn hard_stop_iterative_deepening(&self, depth: Ply, node_count: u64) -> bool {
        if self.is_stopped() {
            return true;
        }
        if self.is_pondering() {
            return false;
        }

        if self
            .node_limit
            .as_ref()
            .is_some_and(|node_limit| node_count >= node_limit.hard_limit)
        {
            return true;
        }

        if self.depth_limit.is_some_and(|max_depth| depth > max_depth) {
            return true;
        }

        if self
            .real_time
            .as_ref()
            .is_some_and(|real_time| real_time.timer.milliseconds() > real_time.hard_time_limit)
        {
            return true;
        }

        return false;
    }

    pub fn is_pondering(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        return self.pondering;

        #[cfg(not(target_arch = "wasm32"))]
        return self.pondering.load(Ordering::SeqCst);
    }

    /// Returns the value of the `stopped` boolean.
    pub fn is_stopped(&self) -> bool {
        #[cfg(target_arch = "wasm32")]
        return self.stopped;

        #[cfg(not(target_arch = "wasm32"))]
        return self.stopped.load(Ordering::SeqCst);
    }

    #[must_use]
    pub fn soft_stop(
        &self,
        node_count: u64,
        best_score: EvalNumber,
        best_move_stability: Ply,
    ) -> bool {
        if self.is_stopped() {
            return true;
        }
        if self.is_pondering() {
            return false;
        }
        if self
            .node_limit
            .as_ref()
            .is_some_and(|node_limit| node_count >= node_limit.soft_limit)
        {
            return true;
        }

        if let Some(ply) = self.mated_in {
            if Search::score_is_checkmate(best_score) {
                if EvalNumber::from(ply) == IMMEDIATE_CHECKMATE_SCORE - best_score.abs() {
                    return true;
                }
            }
        }

        if let Some(real_time) = &self.real_time {
            const BEST_MOVE_STABILITY_MULTIPLIERS: [u64; 8] = [150, 130, 120, 110, 100, 95, 90, 85];
            let multiplier = BEST_MOVE_STABILITY_MULTIPLIERS
                [best_move_stability.min(BEST_MOVE_STABILITY_MULTIPLIERS.len() as u8 - 1) as usize];
            let adjusted_time = (real_time.soft_time_limit * multiplier) / 100;
            return real_time.timer.milliseconds() > adjusted_time.min(real_time.hard_time_limit);
        }

        return false;
    }
}
