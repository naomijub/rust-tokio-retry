# tokio-retry2

Forked from https://github.com/srijs/rust-tokio-retry to keep it up-to-date

Extensible, asynchronous retry behaviours for the ecosystem of [tokio](https://tokio.rs/) libraries.

[![Crates.io](https://img.shields.io/crates/v/tokio-retry2.svg)](https://crates.io/crates/tokio-retry2)
[![dependency status](https://deps.rs/repo/github/naomijub/tokio-retry/status.svg)](https://deps.rs/repo/github/namijub/tokio-retry)
[![codecov](https://codecov.io/gh/naomijub/tokio-retry/branch/main/graph/badge.svg?token=4VMVTZTN8A)](https://codecov.io/gh/naomijub/tokio-retry)

[Documentation](https://docs.rs/tokio-retry2)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tokio-retry2 = { version = "0.5", features = ["jitter", "tracing"] }
```

### Features:
- `jitter`: adds jittery duration to the retry. Mechanism to avoid multiple systems retrying at the same time.
- `tracing`: using `tracing` crate to indicate that a strategy has reached its `max_duration` or `max_delay`.

## Examples

```rust
use tokio_retry2::{Retry, RetryError};
use tokio_retry2::strategy::{ExponentialBackoff, jitter, MaxInterval};

async fn action() -> Result<u64, RetryError<()>> {
    // do some real-world stuff here...
    RetryError::to_transient(())
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .factor(1) // multiplication factor applied to deplay
        .max_delay_millis(100) // set max delay between retries to 500ms
        .max_interval(10000) // set max interval to 10 seconds
        .map(jitter) // add jitter to delays
        .take(3);    // limit to 3 retries

    let result = Retry::spawn(retry_strategy, action).await?;

    Ok(())
}
```

Or, to retry with a notification function:

```rust
use tokio_retry2::{Retry, RetryError};
use tokio_retry2::strategy::{ExponentialBackoff, jitter, MaxInterval};

async fn action() -> Result<u64, RetryError<std::io::Error>> {
    // do some real-world stuff here...
    RetryError::to_permanent(()) // Early exits on this error
}

fn notify(err: &std::io::Error, duration: std::time::Duration) {
    tracing::info!("Error {err:?} occurred at {duration:?}");
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .factor(1) // multiplication factor applied to deplay
        .max_delay_millis(100) // set max delay between retries to 500ms
        .max_interval(10000) // set max interval to 10 seconds
        .map(jitter) // add jitter to delays
        .take(3);    // limit to 3 retries

    let result = Retry::spawn_notify(retry_strategy, action, notify).await?;

    Ok(())
}
```

## Early Exit and Error Handling

Actions must return a `RetryError` that can wrap any other error type. There are 2 `RetryError` error trypes:
- `Permanent`, which receives an error and brakes the retry loop. It can be constructed manually or with auxiliary functions `RetryError::permanent(e: E)`, that returns a `RetryError::Permanent<E>`, or `RetryError::to_permanent(e: E)`, that returns an `Err(RetryError::Permanent<E>)`.
- `Transient`, which is the **Default** error for the loop. It has 2 modes:
    1. `RetryError::transient(e: E)` and `RetryError::to_transient(e: E)`, that return a `RetryError::Transient<E>`, which is an error that triggers the retry strategy.
    2. `RetryError::retry_after(e: E, duration: std::time::Duration)` and `RetryError::to_retry_after(e: E, duration: std::time::Duration)`, that return a `RetryError::Transient<E>`, which is an error that triggers the retry strategy after the specified duration.
- Thet is also the trait `MapErr` that possesses 2 auxiliary functions that map the current function Result to `Result<T, RetryError<E>>`:
    1. `fn map_transient_err(self) -> Result<T, RetryError<E>>;`
    2. `fn map_permanent_err(self) -> Result<T, RetryError<E>>;`
- Using the `?` operator on an `Option` type will always propagate a `RetryError::Transient<E>` with no extra duration.

## Retry Strategies breakdown:

There are 4 backoff strategies:
- `ExponentialBackoff`: base is considered the initial retry interval, so if defined from 500ms, the next retry will happen at 250000ms.
    | attempt | delay |
    |---------|-------|
    | 1       | 500ms |
    | 2       | 250000ms|
- `ExponentialFactorBackoff`: this is a exponential backoff strategy with a base factor. What is exponentially configured is the factor, while the base retry delay is the same. So if a factor 2 is applied to an initial delay off 500ms, the attempts are as follows:
    | attempt | delay |
    |---------|-------|
    | 1       | 500ms |
    | 2       | 1000ms|
    | 3       | 2000ms|
    | 4       | 4000ms|
- `FixedInterval`: in this backoff strategy, a fixed interval is used as constant. so if defined from 500ms, all attempts will happen at 500ms.
    | attempt | delay |
    |---------|-------|
    | 1       | 500ms|
    | 2       | 500ms|
    | 3       | 500ms|
- `FibonacciBackoff`: a Fibonacci backoff strategy is used. so if defined from 500ms, the next retry will happen at 500ms, and the following will be at 1000ms.
    | attempt | delay |
    |---------|-------|
    | 1       | 500ms|
    | 2       | 500ms|
    | 3       | 1000ms|
    | 4       | 1500ms|