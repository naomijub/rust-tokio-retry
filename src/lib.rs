//! This library provides extensible asynchronous retry behaviours
//! for use with the ecosystem of [`tokio`](https://tokio.rs/) libraries.
//!
//! There are 4 backoff strategies:
//! - `ExponentialBackoff`: base is considered the initial retry interval, so if defined from 500ms, the next retry will happen at 250000ms.
//!     | attempt | delay |
//!     |---------|-------|
//!     | 1       | 500ms |
//!     | 2       | 250000ms|
//! - `ExponentialFactorBackoff`: this is a exponential backoff strategy with a base factor. What is exponentially configured is the factor, while the base retry delay is the same. So if a factor 2 is applied to an initial delay off 500ms, the attempts are as follows:
//!     | attempt | delay |
//!     |---------|-------|
//!     | 1       | 500ms |
//!     | 2       | 1000ms|
//!     | 3       | 2000ms|
//!     | 4       | 4000ms|
//!
//! - `FixedInterval`: in this backoff strategy, a fixed interval is used as constant. so if defined from 500ms, all attempts will happen at 500ms.
//!     | attempt | delay |
//!     |---------|-------|
//!     | 1       | 500ms|
//!     | 2       | 500ms|
//!     | 3       | 500ms|
//! - `FibonacciBackoff`: a Fibonacci backoff strategy is used. so if defined from 500ms, the next retry will happen at 500ms, and the following will be at 1000ms.
//!     | attempt | delay |
//!     |---------|-------|
//!     | 1       | 500ms|
//!     | 2       | 500ms|
//!     | 3       | 1000ms|
//!     | 4       | 1500ms|
//!
//! > All strategies can be jittered with the `jitter` feature.
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tokio-retry2 = "0.5"
//! ```
//!
//! # Example
//!
//! ```rust,no_run

//! use tokio_retry2::{Retry, RetryError};
//! use tokio_retry2::strategy::{ExponentialBackoff, MaxInterval};
//!
//! async fn action() -> Result<u64, RetryError<()>> {
//!     // do some real-world stuff here...
//!     RetryError::to_permanent(())
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), RetryError<()>> {
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!     .factor(1) // multiplication factor applied to deplay
//!     .max_delay_millis(100) // set max delay between retries to 500ms
//!     .max_interval(1000) // set max interval to 1 second for all retries
//!     .take(3);    // limit to 3 retries
//!
//! let result = Retry::spawn(retry_strategy, action).await?;
//! // First retry in 10ms, second in 100ms, third in 100ms

//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! One key difference between `tokio-retry2` and `tokio-retry` is the fact that `tokio-retry2`
//! supports early exits from the retry loop based on your error type. This allows you to pattern match
//! your errors and define if you want to continue retrying or not, while `tokio-retry` only supported conditional `RetryIf`.
//! The following functions are helper functions to deal with it:
//!
//! ```rust,no_run
//! use tokio_retry2::{Retry, RetryError};
//! use std::time::Duration;
//!
//! async fn action() -> Result<u64, RetryError<usize>> {
//!     // do some real-world stuff here...
//!     // get and error named `err`
//! #   let err = std::io::ErrorKind::AddrInUse;
//!     match err {
//!         std::io::ErrorKind::NotFound => RetryError::to_permanent(1)?, // equivalent to return Err(RetryError::permanent(2))`;
//!         std::io::ErrorKind::PermissionDenied => {
//!             return Err(RetryError::permanent(2)); // equivalent to `RetryError::to_permanent(2)`
//!         }
//!         std::io::ErrorKind::ConnectionRefused => {
//!             return Err(RetryError::transient(3)); // equivalent to `RetryError::to_transient(3)`
//!         }
//!         std::io::ErrorKind::ConnectionReset => {
//!             return Err(RetryError::retry_after(4, Duration::from_millis(10)));
//!             // equivalent to `RetryError::to_retry_after(4, Duration::from_millis(10))`
//!         }
//!         std::io::ErrorKind::ConnectionAborted =>
//!             // equivalent to `RetryError::to_retry_after(5, Duration::from_millis(15))`
//!             RetryError::to_retry_after(5, Duration::from_millis(15))?,
//!         err => RetryError::to_transient(6)? // equivalent to `return Err(RetryError::transient(6))`
//!     };
//!     Ok(0)
//! }
//! ```
//!
//! ## Features
//! `[jitter]`
//! - `jitter` ranges between 50% and 150% of the strategy delay.
//! - `jitter_range(min: f64, max: f64)` ranges between `min * Duration` and `max * Duration`.
//!
//! To use jitter, add this to your Cargo.toml
//!
//! ```toml
//! [dependencies]
//! tokio-retry2 = { version = "0.5", features = ["jitter"] }
//! ```
//!
//! # Example
//!
//! ## `jitter`
//!
//! ```rust,no_run
//! use tokio_retry2::Retry;
//! use tokio_retry2::strategy::{ExponentialBackoff, jitter, MaxInterval};
//!
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!    .max_interval(10000) // set max interval to 10 seconds
//!    .map(jitter) // add jitter to the retry interval
//!    .take(3);    // limit to 3 retries
//!````
//!
//! ## `jitter_range`
//!
//! ```rust,no_run
//! use tokio_retry2::Retry;
//! use tokio_retry2::strategy::{ExponentialFactorBackoff, jitter_range, MaxInterval};
//!
//! let retry_strategy = ExponentialFactorBackoff::from_millis(10, 2.)
//!    .max_interval(10000) // set max interval to 10 seconds
//!    .map(jitter_range(0.5, 1.2)) // add jitter ranging between 50% and 120% to the retry interval
//!    .take(3);    // limit to 3 retries
//!````
//!
//! ### NOTE:
//! The time spent executing an action does not affect the intervals between
//! retries. Therefore, for long-running functions it's a good idea to set up a deadline,
//! to place an upper bound on the strategy execution time.

#![allow(warnings)]

mod action;
mod condition;
pub(crate) mod error;
mod future;
mod notify;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::Action;
pub use condition::Condition;
pub use error::{Error as RetryError, MapErr};
pub use future::{Retry, RetryIf};
