# 快速開始

本指南將幫助您開始使用 Ichika，從安裝到您的第一個管道。

## 安裝

將 Ichika 添加到您的 `Cargo.toml`：

```toml
[dependencies]
ichika = "0.1"
```

### 功能特性

Ichika 通過功能特性支持不同的異步運行時：

```toml
# 對於 tokio 支持（默認）
ichika = { version = "0.1", features = ["tokio"] }

# 對於 async-std 支持
ichika = { version = "0.1", features = ["async-std"] }

# 同時支持兩個運行時
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## 您的第一個管道

讓我們創建一個處理字符串的簡單管道：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // 定義一個 3 階段管道
    let pool = pipe![
        // 階段 1: 解析字符串為數字
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("解析失敗: {}", e))
        },
        // 階段 2: 將數字加倍
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // 階段 3: 轉換回字符串
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("錯誤: {}", e))
        }
    ]?;

    // 處理一些數據
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // 收集結果
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("結果: {}", result);
        }
    }

    Ok(())
}
```

## 理解基礎

### pipe! 宏

`pipe!` 宏創建一系列處理階段。每個階段：

1. 從上一階段（或初始的 `send()` 調用）接收輸入
2. 在線程池中處理數據
3. 將結果傳遞給下一階段

### 類型傳播

Ichika 自動推斷流經管道的類型：

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### 錯誤處理

每個階段可以返回一個 `Result`，錯誤會自動傳播：

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // 或者適當地處理錯誤
    }
]?;
```

## 下一步

- 了解更多關於 [pipe! 宏](./pipe-macro.md)
- 理解 [ThreadPool trait](./threadpool-trait.md)
- 深入探索 [錯誤處理](./error-handling.md)
- 查看更多 [示例](./examples.md)
