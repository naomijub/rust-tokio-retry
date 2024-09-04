//! This library provides extensible asynchronous retry behaviours
//! for use with the ecosystem of [`tokio`](https://tokio.rs/) libraries.
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tokio-retry2 = "0.4"
//! ```
//!
//! # Example
//!
//! ```rust,no_run

//! use tokio_retry2::Retry;
//! use tokio_retry2::strategy::{ExponentialBackoff, MaxInterval};
//!
//! async fn action() -> Result<u64, ()> {
//!     // do some real-world stuff here...
//!     Err(())
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), ()> {
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!     .factor(1) // multiplication factor applied to deplay
//!     .max_delay_millis(100) // set max delay between retries to 500ms
//!     .max_interval(1000) // set max interval to 1 second for all retries
//!     .take(3);    // limit to 3 retries
//!
//! let result = Retry::spawn(retry_strategy, action).await?;

//! # Ok(())
//! # }
//! ```
//!
//! # Features
//! `[jitter]``
//!
//! To use jitter, add this to your Cargo.toml
//!
//! ```toml
//! [dependencies]
//! tokio-retry2 = { version = "0.4", features = ["jitter"] }
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use tokio_retry2::Retry;
//! use tokio_retry2::strategy::{ExponentialBackoff, jitter, MaxInterval};
//!
//! let retry_strategy = ExponentialBackoff::from_millis(10)
//!    .map(jitter) // add jitter to the retry interval
//!    .max_interval(10000) // set max interval to 10 seconds
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
mod future;
/// Assorted retry strategies including fixed interval and exponential back-off.
pub mod strategy;

pub use action::Action;
pub use condition::Condition;
pub use future::{Retry, RetryIf};
