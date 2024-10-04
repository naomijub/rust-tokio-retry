use tokio::time::Duration;

pub fn jitter(duration: Duration) -> Duration {
    duration.mul_f64(rand::random::<f64>() + 0.5)
}

pub fn jitter_range(min: f64, max: f64) -> impl Fn(Duration) -> Duration {
    move |x| x.mul_f64(rand::random::<f64>() * (max - min) + min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter() {
        let jitter = jitter(Duration::from_millis(100));
        assert!(jitter.as_millis() >= 50);
        assert!(jitter.as_millis() <= 150);
        assert!(jitter.as_millis() != 100);
    }

    #[test]
    fn test_jitter_range() {
        let jitter = jitter_range(0.01, 0.1)(Duration::from_millis(100));
        assert!(jitter.as_millis() >= 1);
        assert!(jitter.as_millis() <= 10);
        assert!(jitter.as_millis() != 100);

        let jitter = jitter_range(0.1, 0.2)(Duration::from_millis(100));
        assert!(jitter.as_millis() >= 10);
        assert!(jitter.as_millis() <= 20);
        assert!(jitter.as_millis() != 100);

        let jitter = jitter_range(0.5, 0.6)(Duration::from_millis(100));
        assert!(jitter.as_millis() >= 50);
        assert!(jitter.as_millis() <= 60);
        assert!(jitter.as_millis() != 100);
    }
}
