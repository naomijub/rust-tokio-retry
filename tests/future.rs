use std::future;
use std::iter::Take;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio_retry2::strategy::ExponentialBackoff;
use tokio_retry2::{Retry, RetryError, RetryIf};

#[tokio::test]
async fn attempts_just_once() {
    use std::iter::empty;
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(empty(), move || {
        cloned_counter.fetch_add(1, Ordering::SeqCst);
        future::ready(Err::<(), RetryError<u64>>(RetryError::transient(42)))
    });
    let res = future.await;

    assert_eq!(res, Err(42));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn attempts_until_max_retries_exceeded() {
    use tokio_retry2::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100).take(2);
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(s, move || {
        cloned_counter.fetch_add(1, Ordering::SeqCst);
        future::ready(Err::<(), RetryError<u64>>(RetryError::transient(42)))
    });
    let res = future.await;

    assert_eq!(res, Err(42));
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn attempts_until_success() {
    use tokio_retry2::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100);
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(s, move || {
        let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
        if previous < 3 {
            future::ready(Err::<(), RetryError<u64>>(RetryError::transient(42)))
        } else {
            future::ready(Ok::<(), RetryError<u64>>(()))
        }
    });
    let res = future.await;

    assert_eq!(res, Ok(()));
    assert_eq!(counter.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn compatible_with_tokio_core() {
    use tokio_retry2::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100);
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(s, move || {
        let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
        if previous < 3 {
            future::ready(Err::<(), RetryError<u64>>(RetryError::transient(42)))
        } else {
            future::ready(Ok::<(), RetryError<u64>>(()))
        }
    });
    let res = future.await;

    assert_eq!(res, Ok(()));
    assert_eq!(counter.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn attempts_retry_only_if_given_condition_is_true() {
    use tokio_retry2::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100).take(5);
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    #[allow(clippy::complexity)]
    let future: RetryIf<Take<FixedInterval>, _, fn(&usize) -> _, fn(&usize, Duration) -> _> =
        RetryIf::spawn(
            s,
            move || {
                let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
                future::ready(Err::<(), RetryError<usize>>(RetryError::transient(
                    previous + 1,
                )))
            },
            |e: &usize| *e < 3,
            |e: &usize, d: Duration| {
                assert!(e == &1 || e == &2);
                assert!(d == Duration::from_millis(0) || d == Duration::from_millis(100));
            },
        );
    let res = future.await;

    assert_eq!(res, Err(3));
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn notify_retry() {
    use tokio_retry2::strategy::FixedInterval;
    let s = FixedInterval::from_millis(100);
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn_notify(
        s,
        move || {
            let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
            if previous < 1 {
                future::ready(Err::<(), RetryError<u64>>(RetryError::transient(42)))
            } else {
                future::ready(Ok::<(), RetryError<u64>>(()))
            }
        },
        message,
    );
    let res = future.await;

    assert_eq!(res, Ok(()));
}

#[tokio::test]
async fn doesnt_attempt_on_permanent() {
    let retry_strategy = ExponentialBackoff::from_millis(10).factor(1).take(3);
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = Retry::spawn(retry_strategy, move || {
        cloned_counter.fetch_add(1, Ordering::SeqCst);
        future::ready(RetryError::to_permanent::<()>(42))
    });
    let res = future.await;

    assert_eq!(res, Err(42));
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn notify_retry_after_duration() {
    let s = tokio_retry2::strategy::FixedInterval::from_millis(100);
    let counter = Arc::new(AtomicUsize::new(0));
    let cloned_counter = counter.clone();
    let future = crate::Retry::spawn_notify(
        s,
        move || {
            let previous = cloned_counter.fetch_add(1, Ordering::SeqCst);
            if previous < 1 {
                future::ready(Err::<(), RetryError<u64>>(RetryError::retry_after(
                    42,
                    Duration::from_millis(100),
                )))
            } else {
                future::ready(Ok::<(), RetryError<u64>>(()))
            }
        },
        message_100ms,
    );
    let res = future.await;

    assert_eq!(res, Ok(()));
}

fn message_100ms(err: &u64, duration: Duration) {
    let msg = format!("err: {}, duration: {:?}", err, duration);
    assert_eq!(msg, "err: 42, duration: 100ms");
}

fn message(err: &u64, duration: Duration) {
    let msg = format!("err: {}, duration: {:?}", err, duration);
    assert_eq!(msg, "err: 42, duration: 0ns");
}
