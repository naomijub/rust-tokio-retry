mod exponential_backoff;
mod exponential_factor_backoff;
mod fibonacci_backoff;
mod fixed_interval;
#[cfg(feature = "jitter")]
mod jitter;
mod max_interval;

pub use self::exponential_backoff::ExponentialBackoff;
pub use self::exponential_factor_backoff::ExponentialFactorBackoff;
pub use self::fibonacci_backoff::FibonacciBackoff;
pub use self::fixed_interval::FixedInterval;
pub use self::max_interval::{MaxInterval, MaxIntervalIterator};

#[cfg(feature = "jitter")]
pub use self::jitter::{jitter, jitter_range};
