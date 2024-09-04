# Changelog

## Version 0.4.0 @ [#29](https://github.com/srijs/rust-tokio-retry/pull/29)

- Adds `Retry::spawn_notify` to notify a retry with the associated `Error` and current `Duration`, PR [#4](https://github.com/naomijub/tokio-retry/pull/4).
- Adds github actions removing obsolete travis.ci.
- Bi-monthly runs CI checking for outdated dependencies.
- Applies const to functions that can be const.
- Adds linting defaults.
- Optional jitter from @fuzzbuck PR [#26](https://github.com/srijs/rust-tokio-retry/pull/26)
- Implements max_interval from @pzmarzly PR [#27](https://github.com/srijs/rust-tokio-retry/pull/27)