# The pipe! Macro

The `pipe!` macro is the core of Ichika. It transforms a sequence of closures into a fully-functional multi-stage processing pipeline.

## Basic Syntax

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... more closures
]?;
```

Each closure represents one processing stage in your pipeline.

## Closure Signatures

Each closure must follow these rules:

1. **Accept exactly one parameter** - the input from the previous stage
2. **Return a type** - this becomes the input to the next stage
3. Be `Clone + Send + 'static` - required for thread pool execution

### Example Signatures

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // Handle the Result
}
```

## Type Inference

Ichika automatically connects the output type of one stage to the input type of the next:

```rust
let pool = pipe![
    |req: String| -> usize {        // Stage 1: String -> usize
        req.len()
    },
    |req: usize| -> String {         // Stage 2: usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // Stage 3: String -> bool
        !req.is_empty()
    }
]?;
```

## Stage Attributes

You can configure individual stages using attributes:

### Thread Pool Configuration

```rust
let pool = pipe![
    #[threads(4)]                    // Use 4 threads for this stage
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // Use 2 threads for this stage
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### Queue Configuration

```rust
let pool = pipe![
    #[queue(100)]                    // Queue capacity of 100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### Named Stages

```rust
let pool = pipe![
    #[name("parser")]                // Name the stage for monitoring
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

// Query task count for a named stage
let count = pool.task_count("parser")?;
```

## Branching Pipelines

You can create conditional branching in your pipeline:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // Handle each branch
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("Number: {}", n),
            Either::Right(s) => format!("String: {}", s),
        }
    }
]?;
```

## Async Stages

With the appropriate feature flag, you can use async stages:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // This runs in the async runtime
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## Global Constraints

You can set global constraints for the entire pipeline:

```rust
let pool = pipe![
    #[global_threads(8)]             // Default thread count for all stages
    #[global_queue(1000)]            // Default queue capacity
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## Complete Example

Here's a more realistic example showing multiple features:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        #[name("parse")]
        #[threads(2)]
        |req: String| -> anyhow::Result<i32> {
            log::info!("Parsing: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        #[threads(4)]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("Processing: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("Formatting: {}", n);
                    format!("Result: {}", n)
                }
                Err(e) => {
                    log::error!("Error: {}", e);
                    format!("Error: {}", e)
                }
            }
        }
    ]?;

    // Monitor thread usage
    println!("Thread usage: {}", pool.thread_usage()?);

    Ok(())
}
```
