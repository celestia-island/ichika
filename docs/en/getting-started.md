# Getting Started

This guide will help you get started with Ichika, from installation to your first pipeline.

## Installation

Add Ichika to your `Cargo.toml`:

```toml
[dependencies]
ichika = "0.1"
```

### Feature Flags

Ichika supports different async runtimes via feature flags:

```toml
# For tokio support (default)
ichika = { version = "0.1", features = ["tokio"] }

# For async-std support
ichika = { version = "0.1", features = ["async-std"] }

# For both runtimes
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## Your First Pipeline

Let's create a simple pipeline that processes strings:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // Define a 3-stage pipeline
    let pool = pipe![
        // Stage 1: Parse string to number
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("Failed to parse: {}", e))
        },
        // Stage 2: Double the number
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // Stage 3: Convert back to string
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("Error: {}", e))
        }
    ]?;

    // Process some data
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // Collect results
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("Result: {}", result);
        }
    }

    Ok(())
}
```

## Understanding the Basics

### The pipe! Macro

The `pipe!` macro creates a chain of processing stages. Each stage:

1. Receives input from the previous stage (or the initial `send()` call)
2. Processes the data in a thread pool
3. Passes the result to the next stage

### Type Propagation

Ichika automatically infers the types flowing through your pipeline:

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### Error Handling

Each stage can return a `Result`, and errors are automatically propagated:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // or handle the error appropriately
    }
]?;
```

## Next Steps

- Learn more about the [pipe! macro](./pipe-macro.md)
- Understand the [ThreadPool trait](./threadpool-trait.md)
- Explore [error handling](./error-handling.md) in depth
- See more [examples](./examples.md)
