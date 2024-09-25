use std::time::Instant;
use tokio::time::Duration;

/// Wraps a strategy, applying `max_interval``, after which strategy will
/// stop retrying.
pub trait MaxInterval: Iterator<Item = Duration> {
    /// Applies a `max_interval` for a strategy. Same as  `max_duration`, but using millis instead of `Duration`.
    fn max_interval(self, max_interval: u64) -> MaxIntervalIterator<Self>
    where
        Self: Sized,
    {
        MaxIntervalIterator {
            iter: self,
            start: Instant::now(),
            max_duration: Duration::from_millis(max_interval),
        }
    }

    /// Applies a `max_duration` for a strategy. In `max_duration` from now,
    /// the strategy will stop retrying. If `max_duration` is passed, the strategy
    /// will stop retrying after `max_duration` is reached.
    fn max_duration(self, max_duration: Duration) -> MaxIntervalIterator<Self>
    where
        Self: Sized,
    {
        MaxIntervalIterator {
            iter: self,
            start: Instant::now(),
            max_duration,
        }
    }
}

impl<I> MaxInterval for I where I: Iterator<Item = Duration> {}

/// A strategy wrapper with applied max_interval,
/// created by [`MaxInterval::max_interval`] function.
#[derive(Debug)]
pub struct MaxIntervalIterator<I> {
    iter: I,
    start: Instant,
    max_duration: Duration,
}

impl<I: Iterator<Item = Duration>> Iterator for MaxIntervalIterator<I> {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.elapsed() > self.max_duration {
            None
        } else {
            self.iter.next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::strategy::FixedInterval;

    #[tokio::test]
    async fn returns_none_after_max_interval_passes() {
        let mut s = FixedInterval::from_millis(10).max_interval(50);
        assert_eq!(s.next(), Some(Duration::from_millis(10)));
        tokio::time::sleep(Duration::from_millis(15)).await;
        assert_eq!(s.next(), Some(Duration::from_millis(10)));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(s.next(), None);
    }

    #[tokio::test]
    async fn returns_none_after_max_duration_passes() {
        let mut s = FixedInterval::from_millis(10).max_duration(Duration::from_millis(50));
        assert_eq!(s.next(), Some(Duration::from_millis(10)));
        tokio::time::sleep(Duration::from_millis(15)).await;
        assert_eq!(s.next(), Some(Duration::from_millis(10)));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(s.next(), None);
    }
}
