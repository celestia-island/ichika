# pipe! 宏

`pipe!` 宏是 Ichika 的核心。它將一系列閉包轉換為功能齊全的多階段處理管道。

## 基本語法

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... 更多閉包
]?;
```

每個閉包代表管道中的一個處理階段。

## 閉包簽名

每個閉包必須遵循以下規則：

1. **接受恰好一個參數** - 來自上一階段的輸入
2. **返回一個類型** - 這將成為下一階段的輸入
3. 是 `Clone + Send + 'static` - 線程池執行所需

### 簽名示例

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // 處理 Result
}
```

## 類型推斷

Ichika 自動將一個階段的輸出類型連接到下一個階段的輸入類型：

```rust
let pool = pipe![
    |req: String| -> usize {        // 階段 1: String -> usize
        req.len()
    },
    |req: usize| -> String {         // 階段 2: usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // 階段 3: String -> bool
        !req.is_empty()
    }
]?;
```

## 階段屬性

您可以使用屬性配置各個階段：

### 線程池配置

```rust
let pool = pipe![
    #[threads(4)]                    // 此階段使用 4 個線程
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // 此階段使用 2 個線程
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### 隊列配置

```rust
let pool = pipe![
    #[queue(100)]                    // 隊列容量為 100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### 命名階段

```rust
let pool = pipe![
    #[name("parser")]                // 命名階段以便監控
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

// 查詢命名階段的任務計數
let count = pool.task_count("parser")?;
```

## 分支管道

您可以在管道中創建條件分支：

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // 處理每個分支
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("數字: {}", n),
            Either::Right(s) => format!("字符串: {}", s),
        }
    }
]?;
```

## 異步階段

啟用相應的功能特性後，您可以使用異步階段：

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // 這在異步運行時中運行
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## 全局約束

您可以為整個管道設置全局約束：

```rust
let pool = pipe![
    #[global_threads(8)]             // 所有階段的默認線程計數
    #[global_queue(1000)]            // 默認隊列容量
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## 完整示例

這是一個展示多個功能的更實際示例：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        #[name("parse")]
        #[threads(2)]
        |req: String| -> anyhow::Result<i32> {
            log::info!("解析: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        #[threads(4)]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("處理: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("格式化: {}", n);
                    format!("結果: {}", n)
                }
                Err(e) => {
                    log::error!("錯誤: {}", e);
                    format!("錯誤: {}", e)
                }
            }
        }
    ]?;

    // 監控線程使用情況
    println!("線程使用情況: {}", pool.thread_usage()?);

    Ok(())
}
```
