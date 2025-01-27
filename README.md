<img src="splash.png" alt="ichika" />

![Crates.io License](https://img.shields.io/crates/l/ichika)
[![Crates.io Version](https://img.shields.io/crates/v/ichika)](https://docs.rs/ichika)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/celestia-island/ichika/test.yml)

## Introduction

This is a helper library for automatically constructing a thread pool that communicates via message pipes. It is based on the `flume` library that is used to communicate between threads.

The name `ichika` comes from the character [ichika](https://bluearchive.wiki/wiki/ichika) in the game [Blue Archive](https://bluearchive.jp/).

> Still in development, the API may change in the future.

## Quick Start

```rust
let (request, response) = ichika::create_async_pool(
    async move |args| {
        Ok(ret.sth()?)
    }
).pipe(
    async move |args| {
        Ok(ret.sth()?)
    }
).run();
```
