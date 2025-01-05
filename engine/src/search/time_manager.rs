use crate::timer::Time;

enum Mode<'a> {
    Depth(u8),
    Time {
        timer: &'a Time,
        hard_time_limit: u64,
        soft_time_limit: u64,
    },
}

pub struct TimeManager<'a> {
    mode: Mode<'a>,
}

impl<'a> TimeManager<'a> {
    pub fn time_limited(timer: &'a Time, hard_time_limit: u64, soft_time_limit: u64) -> Self {
        assert!(hard_time_limit >= soft_time_limit);
        Self {
            mode: Mode::Time {
                timer,
                hard_time_limit,
                soft_time_limit,
            },
        }
    }
    pub fn depth_limited(depth: u8) -> Self {
        Self {
            mode: Mode::Depth(depth),
        }
    }
    pub fn hard_stop_inner_search(&self) -> bool {
        match self.mode {
            Mode::Time {
                timer,
                hard_time_limit,
                ..
            } => timer.milliseconds() > hard_time_limit,
            Mode::Depth(_) => false,
        }
    }
    pub fn hard_stop_iterative_deepening(&self, depth: u8) -> bool {
        match self.mode {
            Mode::Time {
                timer,
                hard_time_limit,
                ..
            } => timer.milliseconds() > hard_time_limit,
            Mode::Depth(max_depth) => depth > max_depth,
        }
    }
    pub fn soft_stop(&self) -> bool {
        match self.mode {
            Mode::Time {
                timer,
                soft_time_limit,
                ..
            } => timer.milliseconds() > soft_time_limit,
            Mode::Depth(_) => false,
        }
    }
}
