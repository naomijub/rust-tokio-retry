# tokio-retry2

Forked from https://github.com/srijs/rust-tokio-retry to keep it up-to-date

Extensible, asynchronous retry behaviours for the ecosystem of [tokio](https://tokio.rs/) libraries.

[![crates](http://meritbadge.herokuapp.com/tokio-retry2)](https://crates.io/crates/tokio-retry2)
[![dependency status](https://deps.rs/repo/github/naomijub/tokio-retry2/status.svg)](https://deps.rs/repo/github/namijub/tokio-retry)


[Documentation](https://docs.rs/tokio-retry2)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tokio-retry = "0.4"
```

## Examples

```rust
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};

async fn action() -> Result<u64, ()> {
    // do some real-world stuff here...
    Err(())
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let retry_strategy = ExponentialBackoff::from_millis(10)
        .map(jitter) // add jitter to delays
        .take(3);    // limit to 3 retries

    let result = Retry::spawn(retry_strategy, action).await?;

    Ok(())
}
```
