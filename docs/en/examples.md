# Examples

This page contains practical examples demonstrating various Ichika features.

## Table of Contents

- [Basic Synchronous Pipeline](#basic-synchronous-pipeline)
- [Basic Asynchronous Pipeline](#basic-asynchronous-pipeline)
- [Error Handling](#error-handling)
- [Graceful Shutdown](#graceful-shutdown)
- [Monitoring Thread Usage](#monitoring-thread-usage)
- [Tuple Payload Pipeline](#tuple-payload-pipeline)

## Basic Synchronous Pipeline

A minimal example showing a simple 2-stage synchronous pipeline:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("Converting '{}' to length", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("Converting length {} back to string", req);
            Ok(req.to_string())
        }
    ]?;

    let inputs = vec!["hello", "world", "ichika"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    loop {
        match pool.recv()? {
            Some(output) => log::info!("Received: {}", output),
            None => break,
        }
    }

    Ok(())
}
```

## Basic Asynchronous Pipeline

Example using async stages with tokio:

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("Stage 1: {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("Stage 2: processing {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("Result: {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Error Handling

Demonstrating error propagation through the pipeline:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("parse")]
        |req: String| -> anyhow::Result<i32> {
            log::info!("Parsing: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("Processing: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("Result: {}", n),
                Err(e) => format!("Error: {}", e),
            }
        }
    ]?;

    let inputs = vec!["42", "100", "invalid", "200"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    std::thread::sleep(std::time::Duration::from_millis(100));

    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Graceful Shutdown

Demonstrating proper cleanup when the pipeline is dropped:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("Processing: {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // Send work
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // Give some time for processing
        std::thread::sleep(Duration::from_millis(200));

        // Pool will shut down gracefully when dropped
        log::info!("Pool going out of scope...");
    }

    log::info!("Pool has shut down gracefully");

    Ok(())
}
```

## Monitoring Thread Usage

Track thread usage and task counts:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("stage1")]
        |req: String| -> usize {
            std::thread::sleep(Duration::from_millis(100));
            req.len()
        },
        #[name("stage2")]
        |req: usize| -> String {
            req.to_string()
        }
    ]?;

    // Send some work
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // Monitor progress
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "Threads: {}, Stage1: {}, Stage2: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("All tasks completed");

    Ok(())
}
```

## Tuple Payload Pipeline

Working with tuple payloads:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}' has length {}", req.0, req.1)
        }
    ]?;

    pool.send("hello".to_string())?;
    pool.send("world".to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Running the Examples

All examples are available in the repository:

```bash
# Run a specific example
cargo run --example basic_sync_chain

# Run with logging
RUST_LOG=info cargo run --example basic_sync_chain

# Run async example
cargo run --example basic_async_chain --features tokio
```

## More Examples

Check the `examples/` directory in the repository for more complete examples:

- `basic_sync_chain.rs` - Synchronous pipeline
- `basic_async_chain.rs` - Asynchronous pipeline
- `error_handling.rs` - Error propagation
- `graceful_shutdown_drop.rs` - Cleanup on drop
- `monitoring_thread_usage.rs` - Monitoring APIs
- `tuple_payload_pipeline.rs` - Complex payload types
- `status_exit_demo.rs` - Status and exit handling
