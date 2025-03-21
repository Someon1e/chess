//! Platform-independent timer

#[cfg(target_arch = "wasm32")]
mod inner {
    pub struct Time {
        start: f64,
    }
    unsafe extern "C" {
        fn time() -> f64;
    }

    impl Time {
        pub fn now() -> Self {
            #[cfg(target_arch = "wasm32")]
            Self {
                start: unsafe { time() },
            }
        }
        pub fn milliseconds(&self) -> u64 {
            let now = unsafe { time() };
            ((now - self.start) * 1000.0) as u64
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod inner {
    use std::time::Instant;

    pub struct Time {
        start: Instant,
    }

    impl Time {
        #[must_use]
        pub fn now() -> Self {
            Self {
                start: Instant::now(),
            }
        }
        #[must_use]
        pub fn milliseconds(&self) -> u64 {
            self.start.elapsed().as_millis().try_into().unwrap()
        }
    }
}

pub use inner::Time;
