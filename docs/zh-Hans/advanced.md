# 高级功能

本节涵盖了充分利用 Ichika 的高级功能和技巧。

## 异步集成

Ichika 支持 `tokio` 和 `async-std` 运行时。通过功能特性启用：

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# 或
ichika = { version = "0.1", features = ["async-std"] }
```

### 异步阶段

无缝混合同步和异步阶段：

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // 同步阶段
        },
        async |req: usize| -> String {
            // 异步阶段 - 在 tokio 运行时中运行
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## 自定义线程创建器

您可以自定义每个阶段如何创建线程：

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // 2MB 栈
            .spawn(|| {
                // 自定义线程逻辑
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## 监控和可观察性

### 线程使用跟踪

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// 获取总线程数
let total_threads = pool.thread_usage()?;

// 获取命名阶段的待处理任务
let pending = pool.task_count("worker")?;

println!("线程: {}, 待处理: {}", total_threads, pending);
```

### 健康检查

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## 资源管理

### 优雅关闭

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// 生成监控线程
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // 监控池健康状况
        thread::sleep(Duration::from_secs(1));
    }
});

// 完成后，将 running 设置为 false
running.store(false, Ordering::Relaxed);
// 当被丢弃时，池将优雅关闭
```

### 内存考虑

每个阶段都有有界队列。根据内存约束调整队列大小：

```rust
let pool = pipe![
    #[queue(100)]   // 内存受限环境的小队列
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  // 高吞吐量阶段的较大队列
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## 管道模式

### 扇出/扇入

并行处理项目并收集结果：

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

### 有状态处理

使用 `Arc<Mutex<T>>` 进行有状态阶段：

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("已处理 {} 个项目", *count);
        req.len()
    }
]?;
```

### 条件路由

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("登录: {}", user),
            Event::Logout(user) => format!("登出: {}", user),
            Event::Message(from, msg) => format!("{}: {}", from, msg),
        }
    }
]?;
```

## 性能调优

### 线程池大小调整

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  // 匹配 CPU 数量
    |req: String| -> usize {
        req.len()
    }
]?;
```

### 批处理

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  // 使用 rayon 进行并行处理
            .map(|s| s.len())
            .collect()
    }
]?;
```

## 测试管道

### 单元测试阶段

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

### 集成测试

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
        }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // 管道应该优雅处理错误
}
```

## 最佳实践

1. **命名您的阶段** 以便更好地监控和调试
2. **使用适当的线程计数** - 不要过度订阅 CPU
3. **设置合理的队列大小** 以限制内存使用
4. **显式处理错误** - 不要默默忽略失败
5. **在生产中监控资源使用**
6. **测试错误路径** - 不仅仅是快乐路径
7. **考虑背压** - 当下游慢时会发生什么？
8. **对 I/O 密集型阶段使用异步**，对 CPU 密集型阶段使用同步
