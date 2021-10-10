All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [v0.2.0](https://github.com/fpagliughi/cooper-rs/compare/v0.1.1..v0.2.0) - 2021-03-28

- Added deferred return from `call()` operations by giving the return `Sender` to the user's closure.
- Async `cast()` operations no longer async. (They don't need to be await'ed)
- Switched to "async-channel" crate for use in all runtimes to come.
- [PR #1](https://github.com/fpagliughi/cooper-rs/pull/1) Minor formatting and type fixes
- [PR #2](https://github.com/fpagliughi/cooper-rs/pull/2) Supports the `tokio` runtime

## [v0.1.1](https://github.com/fpagliughi/cooper-rs/compare/v0.1.0..v0.1.1) - 2021-03-27

- Added `ThreadedActor<>` with example

## v0.1.0 - 2021-03-27

- Initial async `Actor<>` with some examples.

