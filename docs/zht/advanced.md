# 高級功能

本節涵蓋了充分利用 Ichika 的高級功能和技巧。

## 異步集成

Ichika 支持 `tokio` 和 `async-std` 運行時。通過功能特性啟用：

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# 或
ichika = { version = "0.1", features = ["async-std"] }
```

### 異步階段

無縫混合同步和異步階段：

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // 同步階段
        },
        async |req: usize| -> String {
            // 異步階段 - 在 tokio 運行時中運行
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## 自定義線程創建器

您可以自定義每個階段如何創建線程：

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // 2MB 棧
            .spawn(|| {
                // 自定義線程邏輯
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## 監控和可觀察性

### 線程使用跟踪

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// 獲取總線程數
let total_threads = pool.thread_usage()?;

// 獲取命名階段的待處理任務
let pending = pool.task_count("worker")?;

println!("線程: {}, 待處理: {}", total_threads, pending);
```

### 健康檢查

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## 資源管理

### 優雅關閉

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// 生成監控線程
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // 監控池健康狀況
        thread::sleep(Duration::from_secs(1));
    }
});

// 完成後，將 running 設置為 false
running.store(false, Ordering::Relaxed);
// 當被丟棄時，池將優雅關閉
```

### 內存考慮

每個階段都有有界隊列。根據內存約束調整隊列大小：

```rust
let pool = pipe![
    #[queue(100)]   // 內存受限環境的小隊列
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  // 高吞吐量階段的較大隊列
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## 管道模式

### 扇出/扇入

並行處理項目並收集結果：

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

### 有狀態處理

使用 `Arc<Mutex<T>>` 進行有狀態階段：

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("已處理 {} 個項目", *count);
        req.len()
    }
]?;
```

### 條件路由

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("登錄: {}", user),
            Event::Logout(user) => format!("登出: {}", user),
            Event::Message(from, msg) => format!("{}: {}", from, msg),
        }
    }
]?;
```

## 性能調優

### 線程池大小調整

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  // 匹配 CPU 數量
    |req: String| -> usize {
        req.len()
    }
]?;
```

### 批處理

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  // 使用 rayon 進行並行處理
            .map(|s| s.len())
            .collect()
    }
]?;
```

## 測試管道

### 單元測試階段

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

### 集成測試

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
        }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // 管道應該優雅處理錯誤
}
```

## 最佳實踐

1. **命名您的階段** 以便更好地監控和調試
2. **使用適當的線程計數** - 不要過度訂閱 CPU
3. **設置合理的隊列大小** 以限制內存使用
4. **顯式處理錯誤** - 不要默默忽略失敗
5. **在生產中監控資源使用**
6. **測試錯誤路徑** - 不僅僅是快樂路徑
7. **考慮背壓** - 當下游慢時會發生什麼？
8. **對 I/O 密集型階段使用異步**，對 CPU 密集型階段使用同步
