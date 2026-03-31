# 示例

本頁包含演示各種 Ichika 功能的實際示例。

## 目錄

- [基本同步管道](#基本同步管道)
- [基本異步管道](#基本異步管道)
- [錯誤處理](#錯誤處理)
- [優雅關閉](#優雅關閉)
- [監控線程使用](#監控線程使用)
- [元組負載管道](#元組負載管道)

## 基本同步管道

展示簡單 2 階段同步管道的最小示例：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("將 '{}' 轉換為長度", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("將長度 {} 轉換回字符串", req);
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

## 基本異步管道

使用 tokio 的異步階段示例：

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("階段 1: {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("階段 2: 處理 {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("結果: {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## 錯誤處理

演示錯誤在管道中的傳播：

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
            log::info!("處理: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("結果: {}", n),
                Err(e) => format!("錯誤: {}", e),
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

## 優雅關閉

演示當管道被丟棄時的正確清理：

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("處理: {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // 發送工作
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // 給一些時間處理
        std::thread::sleep(Duration::from_millis(200));

        // 池將被丟棄並優雅關閉
        log::info!("池超出範圍...");
    }

    log::info!("池已優雅關閉");

    Ok(())
}
```

## 監控線程使用

跟踪線程使用和任務計數：

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

    // 發送一些工作
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // 監控進度
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "線程: {}, 階段1: {}, 階段2: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("所有任務已完成");

    Ok(())
}
```

## 元組負載管道

處理元組負載：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}' 的長度為 {}", req.0, req.1)
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

## 運行示例

存儲庫中提供了所有示例：

```bash
# 運行特定示例
cargo run --example basic_sync_chain

# 使用日誌記錄運行
RUST_LOG=info cargo run --example basic_sync_chain

# 運行異步示例
cargo run --example basic_async_chain --features tokio
```

## 更多示例

查看存儲庫中的 `examples/` 目錄以獲取更多完整示例：

- `basic_sync_chain.rs` - 同步管道
- `basic_async_chain.rs` - 異步管道
- `error_handling.rs` - 錯誤傳播
- `graceful_shutdown_drop.rs` - 刪除時清理
- `monitoring_thread_usage.rs` - 監控 API
- `tuple_payload_pipeline.rs` - 複雜負載類型
- `status_exit_demo.rs` - 狀態和退出處理
