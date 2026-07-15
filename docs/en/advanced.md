# Advanced Features

This section covers advanced features and techniques for getting the most out of Ichika.

## Async Integration

Ichika supports both `tokio` and `async-std` runtimes. Enable with feature flags:

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# or
ichika = { version = "0.1", features = ["async-std"] }
```

### Async Stages

Mix sync and async stages seamlessly:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // Sync stage
        },
        async |req: usize| -> String {
            // Async stage - runs in tokio runtime
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## Custom Thread Creators

You can customize how threads are created for each stage:

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // 2MB stack
            .spawn(|| {
                // Custom thread logic
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## Monitoring and Observability

### Thread Usage Tracking

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// Get total thread count
let total_threads = pool.thread_usage()?;

// Get pending tasks for a named stage
let pending = pool.task_count("worker")?;

println!("Threads: {}, Pending: {}", total_threads, pending);
```

### Health Checks

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## Resource Management

### Graceful Shutdown

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// Spawn a monitoring thread
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // Monitor pool health
        thread::sleep(Duration::from_secs(1));
    }
});

// When done, set running to false
running.store(false, Ordering::Relaxed);
// Pool will shut down gracefully when dropped
```

### Memory Considerations

Each stage has a bounded queue. Adjust queue sizes based on your memory constraints:

```rust
let pool = pipe![
    #[queue(100)]   // Small queue for memory-constrained environments
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  // Larger queue for high-throughput stages
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## Pipeline Patterns

### Fan-Out / Fan-In

Process items in parallel and collect results:

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<String> {
        req.into_iter()
            .filter(|s| !s.is_empty())
            .collect()
    },
    |req: Vec<String>| -> usize {
        req.len()
    }
]?;
```

### Stateful Processing

Use `Arc<Mutex<T>>` for stateful stages:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("Processed {} items", *count);
        req.len()
    }
]?;
```

### Conditional Routing

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("Login: {}", user),
            Event::Logout(user) => format!("Logout: {}", user),
            Event::Message(from, msg) => format!("{}: {}", from, msg),
        }
    }
]?;
```

## Performance Tuning

### Thread Pool Sizing

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  // Match CPU count
    |req: String| -> usize {
        req.len()
    }
]?;
```

### Batch Processing

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  // Use rayon for parallel processing
            .map(|s| s.len())
            .collect()
    }
]?;
```

## Testing Pipelines

### Unit Testing Stages

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline() {
        let pool = pipe![
            |req: String| -> usize { req.len() },
            |req: usize| -> String { req.to_string() }
        ].unwrap();

        pool.send("test".to_string()).unwrap();
        let result = pool.recv().unwrap().unwrap();
        assert_eq!(result, "4");
    }
}
```

### Integration Testing

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
        }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // Pipeline should handle errors gracefully
}
```

## Best Practices

1. **Name your stages** for better monitoring and debugging
2. **Use appropriate thread counts** - don't oversubscribe your CPU
3. **Set reasonable queue sizes** to bound memory usage
4. **Handle errors explicitly** - don't silently ignore failures
5. **Monitor resource usage** in production
6. **Test error paths** - not just happy paths
7. **Consider backpressure** - what happens when downstream is slow?
8. **Use async for I/O-bound** stages, sync for CPU-bound stages
