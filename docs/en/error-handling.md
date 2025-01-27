# Error Handling & Retry

Ichika provides robust error handling with built-in retry semantics for transient failures.

## Error Propagation

Errors naturally flow through the pipeline using `Result` types:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
        let n = req?;
        Ok(n * 2)
    },
    |req: anyhow::Result<i32>| -> String {
        match req {
            Ok(n) => format!("Result: {}", n),
            Err(e) => format!("Error: {}", e),
        }
    }
]?;
```

### Type Transformation

When a stage returns a `Result`, the next stage receives that `Result`:

```rust
|req: String| -> anyhow::Result<usize> { ... }  // Returns Result
|req: anyhow::Result<usize>| -> usize {         // Receives Result
    req.unwrap()
}
```

## Retry Semantics

Ichika provides automatic retry for operations that may fail transiently.

### Basic Retry

Use the `retry` function to retry an operation:

```rust
use ichika::retry;

let result = retry(|| {
    // Operation that might fail
    Ok::<_, anyhow::Error>(42)
})?;
```

### Retry with Policy

Control retry behavior with a `RetryPolicy`:

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // Operation with custom retry policy
    Ok::<_, anyhow::Error>(42)
})?;
```

### RetryPolicy Options

```rust
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: usize,

    /// Initial backoff duration (exponential backoff is applied)
    pub backoff: Duration,

    /// Maximum backoff duration
    pub max_backoff: Duration,

    /// Whether to use jitter in backoff calculation
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            jitter: true,
        }
    }
}
```

## Using Retry in Pipelines

### Retry Within a Stage

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // Retry the fetch operation
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // Simulated fetch that might fail
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("Network error"))
                } else {
                    Ok(format!("Fetched: {}", req))
                }
            }
        )
    }
]?;
```

### Retry at Pipeline Level

For more control, handle retry at the caller level:

```rust
fn process_with_retry(pool: &impl ThreadPool<Request = String, Response = String>, input: String) -> anyhow::Result<String> {
    retry_with(
        RetryPolicy {
            max_attempts: 5,
            backoff: Duration::from_millis(50),
            ..Default::default()
        },
        || {
            pool.send(input.clone())?;
            match pool.recv()? {
                Some(result) => Ok(result),
                None => Err(anyhow::anyhow!("Pipeline terminated")),
            }
        }
    )
}
```

## Error Recovery Strategies

### Fallback Values

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  // Default to 0 on error
    }
]?;
```

### Error Aggregation

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<anyhow::Result<i32>> {
        req.into_iter()
            .map(|s| s.parse::<i32>().map_err(Into::into))
            .collect()
    },
    |req: Vec<anyhow::Result<i32>>| -> (i32, usize) {
        let (sum, errors) = req.into_iter().fold(
            (0, 0),
            |(sum, errs), r| match r {
                Ok(n) => (sum + n, errs),
                Err(_) => (sum, errs + 1),
            },
        );
        (sum, errors)
    }
]?;
```

### Circuit Breaker Pattern

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Circuit breaker is open"));
        }
        // Process request
        Ok(format!("Processed: {}", req))
    }
]?;
```

## Complete Example

Here's a comprehensive example showing error handling with retry:

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("Invalid input: {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // Simulate transient failure
            if n % 3 == 0 {
                Err(anyhow::anyhow!("Transient error"))
            } else {
                Ok(n * 2)
            }
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("Success: {}", n),
                Err(e) => format!("Failed: {}", e),
            }
        }
    ]?;

    // Send various inputs
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // Collect results
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Best Practices

1. **Use `anyhow::Result`** for flexible error handling
2. **Set appropriate retry limits** to avoid infinite loops
3. **Use exponential backoff** for network operations
4. **Log errors appropriately** for debugging
5. **Consider circuit breakers** for external service calls
6. **Make errors informative** - include context about what failed
