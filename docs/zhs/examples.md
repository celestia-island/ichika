# 示例

本页包含演示各种 Ichika 功能的实际示例。

## 目录

- [基本同步管道](#基本同步管道)
- [基本异步管道](#基本异步管道)
- [错误处理](#错误处理)
- [优雅关闭](#优雅关闭)
- [监控线程使用](#监控线程使用)
- [元组负载管道](#元组负载管道)

## 基本同步管道

展示简单 2 阶段同步管道的最小示例：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("将 '{}' 转换为长度", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("将长度 {} 转换回字符串", req);
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
            Some(output) => log::info!("收到: {}", output),
            None => break,
        }
    }

    Ok(())
}
```

## 基本异步管道

使用 tokio 的异步阶段示例：

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("阶段 1: {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("阶段 2: 处理 {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("结果: {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## 错误处理

演示错误在管道中的传播：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("parse")]
        |req: String| -> anyhow::Result<i32> {
            log::info!("解析: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("处理: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("结果: {}", n),
                Err(e) => format!("错误: {}", e),
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

## 优雅关闭

演示当管道被丢弃时的正确清理：

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("处理: {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // 发送工作
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // 给一些时间处理
        std::thread::sleep(Duration::from_millis(200));

        // 池将被丢弃并优雅关闭
        log::info!("池超出范围...");
    }

    log::info!("池已优雅关闭");

    Ok(())
}
```

## 监控线程使用

跟踪线程使用和任务计数：

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

    // 发送一些工作
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // 监控进度
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "线程: {}, 阶段1: {}, 阶段2: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("所有任务已完成");

    Ok(())
}
```

## 元组负载管道

处理元组负载：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}' 的长度为 {}", req.0, req.1)
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

## 运行示例

存储库中提供了所有示例：

```bash
# 运行特定示例
cargo run --example basic_sync_chain

# 使用日志记录运行
RUST_LOG=info cargo run --example basic_sync_chain

# 运行异步示例
cargo run --example basic_async_chain --features tokio
```

## 更多示例

查看存储库中的 `examples/` 目录以获取更多完整示例：

- `basic_sync_chain.rs` - 同步管道
- `basic_async_chain.rs` - 异步管道
- `error_handling.rs` - 错误传播
- `graceful_shutdown_drop.rs` - 删除时清理
- `monitoring_thread_usage.rs` - 监控 API
- `tuple_payload_pipeline.rs` - 复杂负载类型
- `status_exit_demo.rs` - 状态和退出处理
