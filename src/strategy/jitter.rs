use tokio::time::Duration;

pub fn jitter(duration: Duration) -> Duration {
    duration.mul_f64(rand::random::<f64>())
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
}
