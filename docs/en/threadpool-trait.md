# ThreadPool Trait

The `ThreadPool` trait defines the interface for all pipeline pools created by the `pipe!` macro.

## Trait Definition

```rust
pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&self, req: Self::Request) -> Result<()>;
    fn recv(&self) -> Result<Option<Self::Response>>;

    fn thread_usage(&self) -> Result<usize>;
    fn task_count(&self, id: impl ToString) -> Result<usize>;
}
```

## Methods

### send

Sends a request to the pipeline for processing.

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**Parameters:**
- `req` - The request to send, must match the pipeline's input type

**Returns:**
- `Result<()>` - Ok if successfully queued, Err if the send fails

**Example:**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

Receives the next processed result from the pipeline.

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**Returns:**
- `Ok(Some(response))` - A processed result
- `Ok(None)` - The pipeline has terminated
- `Err(...)` - An error occurred while receiving

**Example:**

```rust
loop {
    match pool.recv()? {
        Some(result) => println!("Got: {}", result),
        None => break,
    }
}
```

### thread_usage

Returns the current number of threads in use by the pipeline.

```rust
fn thread_usage(&self) -> Result<usize>
```

**Returns:**
- The total number of active threads across all stages

**Example:**

```rust
println!("Active threads: {}", pool.thread_usage()?);
```

### task_count

Returns the number of pending tasks for a named stage.

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**Parameters:**
- `id` - The stage name (as set by `#[name(...)]` attribute)

**Returns:**
- The number of tasks waiting in that stage's queue

**Example:**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("Parser queue depth: {}", pool.task_count("parser")?);
```

## Type Parameters

### Request

The input type for the pipeline. This is the type accepted by the first stage.

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

The output type of the pipeline. This is the type returned by the last stage.

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## Lifecycle

The pipeline follows this lifecycle:

1. **Created** - The `pipe!` macro returns a new pool
2. **Active** - You can `send()` requests and `recv()` results
3. **Draining** - When dropped, the pool finishes processing pending tasks
4. **Terminated** - `recv()` returns `None` when the pool is shut down

## Graceful Shutdown

When the pool is dropped, it:

1. Stops accepting new requests
2. Finishes processing all queued tasks
3. Shuts down all thread pools gracefully

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // pool goes out of scope and shuts down gracefully
}
```

## Monitoring

Use the monitoring methods to track pipeline health:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("stage1")]
        |req: String| -> usize { req.len() },
        #[name("stage2")]
        |req: usize| -> String { req.to_string() }
    ]?;

    // Send work
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // Monitor progress
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "Threads: {}, Stage1 pending: {}, Stage2 pending: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
```
