<p align="center"><img src="https://raw.githubusercontent.com/celestia-island/ichika/master/docs/logo.webp" alt="Ichika" width="240" /></p>

[![License: SySL](https://img.shields.io/badge/license-SySL%201.0-blue)](./LICENSE)
[![Crates.io Version](https://img.shields.io/crates/v/ichika)](https://crates.io/crates/ichika)
[![docs.rs](https://docs.rs/ichika/badge.svg)](https://docs.rs/ichika)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/celestia-island/ichika/test.yml)

## Introduction

This is a helper library for automatically constructing a thread pool that communicates via message pipes. It is based on the `flume` library that is used to communicate between threads.

The name `ichika` comes from the character [ichika](https://bluearchive.wiki/wiki/ichika) in the game [Blue Archive](https://bluearchive.jp/).

> Still in development, the API may change in the future.

## Quick Start

A pipeline is a chain of closures: each stage receives the previous stage's
output and returns the next stage's input (wrapped in `Ok`). The `pipe!` macro
wires them together with a thread pool communicating over `flume` channels.

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // A 2-stage pipeline: String -> usize -> String
    let pool = pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    for input in ["hello", "ichika", "pipe"] {
        pool.send(input.to_string())?;
    }

    // Give the background pool time to drain its work.
    std::thread::sleep(std::time::Duration::from_millis(200));

    // `recv` is non-blocking and returns `None` when nothing is ready.
    while let Some(output) = pool.recv()? {
        println!("{output}");
    }

    Ok(())
}
```

Stages may also be `async` (under the `tokio` feature, which is on by default):

```rust
use ichika::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        async |req: String| -> usize {
            tokio::task::yield_now().await;
            Ok(req.len())
        },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("ichika".to_string())?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    while let Some(output) = pool.recv()? {
        println!("{output}");
    }

    Ok(())
}
```

## Features

- [x] `async`, including `tokio` and `async-std`.
- [x] Named task.
- [x] Limit steps' thread usage.
- [x] Multiple target `match` with any depth.
- [x] Error handle target `catch`.
- [x] Retryable target `retry` with timeout parameter.

## License

Licensed under the [Synthetic Source License (SySL), Version 1.0](./LICENSE).
