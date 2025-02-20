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
use ichika::prelude::*;

let pool = pipe! [
  async |(name: String, url: String)| -> anyhow::Result<(String, bytes::Bytes))> {
    Ok((name, id, reqwest::get(url).await?.bytes().await?))
  },
  |(name: String, buffer: bytes::Bytes)| -> anyhow::Result<(String, bytes::Bytes)> {
    let mut decoder = flate2::read::GzDecoder::new();
    let mut ret = vec![];
    decoder.read_to_end(&mut ret)?;
    Ok((name, ret.into()))
  },
  async |(name: String, data: bytes::Bytes)| -> anyhow::Result<()> {
    tokio::fs::write(
      format!("./{name}.dat"),
      &data
    );
    Ok(())
  }
]?;

for i in 0..10 {
  pool.send(("sth", vec![0; 32], "https://example.com".to_string()));
}

for i in 0..10 {
  pool.recv().await?;
}
```

## TODO

- [x] `async`, including `tokio` and `async-std`.
- [ ] Named task.
- [ ] Limit steps' thread usage.
- [ ] Multiple target `match` with any depth.
- [ ] Error handle target `catch`.
- [ ] Retryable target `retry` with timeout parameter.
