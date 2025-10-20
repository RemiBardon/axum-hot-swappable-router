# `axum` hot-swappable router

An [`axum`] router that can be replaced at runtime with no downtime.

## High level features

- **No downtime.** `axum::serve` takes ownership of `axum::Router`s.
  To change the exposed routes at runtime, you need to stop the TCP listener
  then create it again, resulting in downtime. This library supports
  hot-swapping the `axum::Router` with no downtime.
- **No connection termination.** In-flight requests will finish properly with
  the previous router and its state, while new requests will be served by the
  new one.
- **Minimal overhead.** This library uses [`arc-swap`] to avoid the overhead of
  using locks. When serving thousands of requests a second, this would be
  noticeable.

## Performance

Same as a regular `axum::Router`. See [benchmarks/RESULT.txt](./benchmarks/RESULT.txt).

Note that to prevent connection terminations your app’s state will be
duplicated for some time (as long as the previous router has open connections).
Watch you RAM usage if your app state is very large, or consider sharing some
data across app state instances.

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure everything is implemented in 100% safe Rust.

## Minimum supported Rust version

This crate depends on [`axum`], which has a MSRV of 1.78.

## Examples

The examples folder contains various examples of how to use axum.
For full-fledged examples, check out [`prose-im/prose-pod-api`] and [`prose-im/prose-pod-server/api`].

## License

This project is licensed under the [Apache-2.0 license].

[`axum`]: https://crates.io/crates/axum "axum - crates.io: Rust Package Registry"
[`arc-swap`]: https://crates.io/crates/arc-swap "arc-swap - crates.io: Rust Package Registry"
[`prose-im/prose-pod-api`]: https://github.com/prose-im/prose-pod-api "prose-im/prose-pod-api: Prose Pod API server. REST API used for administration and management."
[`prose-im/prose-pod-api`]: https://github.com/prose-im/prose-pod-server/tree/master/api "prose-im/prose-pod-server: Prose Pod server source code. Depends on the official Prosody XMPP server and extended for Prose requirements."
[Apache-2.0 license]: http://www.apache.org/licenses/LICENSE-2.0
