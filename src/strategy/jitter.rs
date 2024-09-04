use tokio::time::Duration;

pub fn jitter(duration: Duration) -> Duration {
    duration.mul_f64(rand::random::<f64>())
}
