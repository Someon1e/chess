#[cfg(target_arch = "wasm32")]
pub mod timer {
    pub struct Time {
        start: f64,
    }
    extern "C" {
        fn time() -> f64;
    }

    impl Time {
        pub fn now() -> Self {
            #[cfg(target_arch = "wasm32")]
            Self {
                start: unsafe { time() },
            }
        }
        pub fn miliseconds(&self) -> u128 {
            let now = unsafe { time() };
            ((now - self.start) * 1000.0) as u128
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub mod timer {
    use std::time::Instant;

    pub struct Time {
        start: Instant,
    }

    impl Time {
        pub fn now() -> Self {
            Self {
                start: Instant::now(),
            }
        }
        pub fn miliseconds(&self) -> u128 {
            self.start.elapsed().as_millis()
        }
    }
}
