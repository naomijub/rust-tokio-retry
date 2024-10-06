use std::iter::Iterator;
use tokio::time::Duration;

/// A retry strategy driven by exponential factor back-off.
/// Duration is capped at a maximum value of `u32::MAX millis = 4294967295 ms` ~49 days.
///
/// The power corresponds to the number of past attempts.
#[derive(Debug, Clone)]
pub struct ExponentialFactorBackoff {
    base: u64,
    factor: f64,
    base_factor: f64,
    max_delay: Option<Duration>,
}

impl ExponentialFactorBackoff {
    /// Constructs a new exponential factor back-off strategy,
    /// given a initial duration in milliseconds and base factor.
    /// Starting factor is `1.0` to use `initial_delay` as the base.
    ///
    /// The resulting duration is calculated by taking the base factor to the `n`-th power
    /// and multiply it by the initial delay in milliseconds, where `n` denotes the number of past attempts.
    pub const fn from_millis(initial_delay: u64, base_factor: f64) -> Self {
        ExponentialFactorBackoff {
            base: initial_delay,
            factor: 1f64,
            max_delay: None,
            base_factor,
        }
    }

    /// Constructs a new exponential factor back-off strategy,
    /// given a base factor. The initial delay is set to `500`.
    /// Starting factor is `1.0` to use `initial_delay` as the base.
    ///
    /// The resulting duration is calculated by taking the base factor to the `n`-th power
    /// and multiply it by the 500 milliseconds, where `n` denotes the number of past attempts.
    pub const fn from_factor(base_factor: f64) -> Self {
        ExponentialFactorBackoff {
            base: 500,
            factor: 1f64,
            max_delay: None,
            base_factor,
        }
    }

    /// A initial delay in milliseconds for the strategy.
    ///
    /// Default initial_delay is `500`.
    pub const fn initial_delay(mut self, initial_delay: u64) -> ExponentialFactorBackoff {
        self.base = initial_delay;
        self
    }

    /// Apply a maximum delay. No single retry delay will be longer than this `Duration`.
    pub const fn max_delay(mut self, duration: Duration) -> ExponentialFactorBackoff {
        self.max_delay = Some(duration);
        self
    }

    /// Apply a maximum delay. No single retry delay will be longer than this `Duration::from_millis`.
    pub const fn max_delay_millis(mut self, duration: u64) -> ExponentialFactorBackoff {
        self.max_delay = Some(Duration::from_millis(duration));
        self
    }
}

impl Iterator for ExponentialFactorBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        // set delay duration by applying factor
        let duration = (self.base as f64) * self.factor;

        let duration = if duration > u32::MAX as f64 {
            Duration::from_millis(u32::MAX as u64)
        } else {
            Duration::from_millis(duration as u64)
        };

        // check if we reached max delay
        if let Some(ref max_delay) = self.max_delay {
            if duration > *max_delay {
                #[cfg(feature = "tracing")]
                tracing::warn!("`max_delay` for strategy reached");
                return Some(*max_delay);
            }
        }

        let next = self.factor * self.base_factor;
        self.factor = next;

        Some(duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_some_exponential_base_10() {
        let mut s = ExponentialFactorBackoff::from_millis(10, 10.);

        assert_eq!(s.next(), Some(Duration::from_millis(10)));
        assert_eq!(s.next(), Some(Duration::from_millis(100)));
        assert_eq!(s.next(), Some(Duration::from_millis(1000)));
    }

    #[test]
    fn returns_some_exponential_base_4() {
        let mut s = ExponentialFactorBackoff::from_millis(10, 4.);

        assert_eq!(s.next(), Some(Duration::from_millis(10)));
        assert_eq!(s.next(), Some(Duration::from_millis(40)));
        assert_eq!(s.next(), Some(Duration::from_millis(160)));
    }

    #[test]
    fn returns_some_exponential_base_2() {
        let mut s = ExponentialFactorBackoff::from_millis(10, 2.);

        assert_eq!(s.next(), Some(Duration::from_millis(10)));
        assert_eq!(s.next(), Some(Duration::from_millis(20)));
        assert_eq!(s.next(), Some(Duration::from_millis(40)));
        assert_eq!(s.next(), Some(Duration::from_millis(80)));
    }

    #[test]
    fn saturates_at_maximum_value() {
        let mut s = ExponentialFactorBackoff::from_millis((u32::MAX - 1) as u64, 2.0);

        assert_eq!(s.next(), Some(Duration::from_millis((u32::MAX - 1) as u64)));
        assert_eq!(s.next(), Some(Duration::from_millis(u32::MAX as u64)));
        assert_eq!(s.next(), Some(Duration::from_millis(u32::MAX as u64)));
    }

    #[test]
    fn can_use_factor_to_get_seconds() {
        let one_second = 1000;
        let mut s = ExponentialFactorBackoff::from_factor(2.).initial_delay(one_second);

        assert_eq!(s.next(), Some(Duration::from_secs(1)));
        assert_eq!(s.next(), Some(Duration::from_secs(2)));
        assert_eq!(s.next(), Some(Duration::from_secs(4)));
        assert_eq!(s.next(), Some(Duration::from_secs(8)));
    }

    #[test]
    fn stops_increasing_at_max_delay() {
        let mut s =
            ExponentialFactorBackoff::from_millis(1, 2.).max_delay(Duration::from_millis(4));

        assert_eq!(s.next(), Some(Duration::from_millis(1)));
        assert_eq!(s.next(), Some(Duration::from_millis(2)));
        assert_eq!(s.next(), Some(Duration::from_millis(4)));
        assert_eq!(s.next(), Some(Duration::from_millis(4)));
    }

    #[test]
    fn returns_max_when_max_less_than_base() {
        let mut s =
            ExponentialFactorBackoff::from_millis(20, 10.).max_delay(Duration::from_millis(10));

        assert_eq!(s.next(), Some(Duration::from_millis(10)));
        assert_eq!(s.next(), Some(Duration::from_millis(10)));
    }

    #[test]
    fn demo() {
        let mut s = ExponentialFactorBackoff::from_millis(500, 2.);

        assert_eq!(s.next(), Some(Duration::from_millis(500)));
        assert_eq!(s.next(), Some(Duration::from_millis(1000)));
        assert_eq!(s.next(), Some(Duration::from_millis(2000)));
        assert_eq!(s.next(), Some(Duration::from_millis(4000)));
    }
}
