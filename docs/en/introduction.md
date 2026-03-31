# Introduction

**Ichika** is a Rust procedural macro library for building thread pool based pipelines with automatic error handling, retry semantics, and graceful shutdown support.

## Overview

Ichika provides a powerful `pipe!` macro that allows you to define complex multi-stage processing pipelines where each stage runs in its own thread pool. The macro handles all the boilerplate of creating thread pools, setting up communication channels, and coordinating between stages.

## Key Features

- **Declarative Pipeline Syntax**: Define complex processing pipelines using a clean, expressive macro syntax
- **Automatic Thread Pool Management**: Each stage gets its own dedicated thread pool
- **Error Propagation**: Built-in error handling with `Result` types throughout the pipeline
- **Retry Semantics**: Configurable retry policies for handling transient failures
- **Async Runtime Agnostic**: Works with both `tokio` and `async-std`
- **Graceful Shutdown**: Proper cleanup when the pipeline is dropped
- **Monitoring**: Built-in thread usage statistics and task counting

## A Simple Example

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // Create a simple 2-stage pipeline
    let pool = pipe![
        |req: String| -> usize {
            Ok(req.len())
        },
        |req: usize| -> String {
            Ok(req.to_string())
        }
    ]?;

    // Send some requests
    pool.send("hello".to_string())?;
    pool.send("world".to_string())?;

    // Collect results
    while let Some(result) = pool.recv()? {
        println!("Got: {}", result);
    }

    Ok(())
}
```

## Use Cases

Ichika is particularly useful for:

- **Data Processing Pipelines**: Multi-stage data transformation workflows
- **API Request Handling**: Processing requests through multiple validation/transformation stages
- **Event Processing**: Building event-driven systems with staged processing
- **Batch Jobs**: Parallel processing with configurable concurrency per stage
- **Microservices**: Internal service communication with bounded queues

## Design Philosophy

Ichika follows these principles:

1. **Safety First**: Leverage Rust's type system for compile-time guarantees
2. **Ergonomic API**: Minimize boilerplate while maintaining flexibility
3. **Zero Cost Abstractions**: No runtime overhead beyond what's necessary
4. **Explicit Control**: Give users fine-grained control over thread pools and queues

## Project Status

Ichika is currently in active development. The API may change between versions, but we strive to maintain backward compatibility whenever possible.

## License

Ichika is licensed under the MIT License. See [LICENSE](https://github.com/celestia-island/ichika/blob/master/LICENSE) for details.
