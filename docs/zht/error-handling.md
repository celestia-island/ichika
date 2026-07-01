# 錯誤處理與重試

Ichika 提供強大的錯誤處理，內置重試語義用於處理瞬態故障。

## 錯誤傳播

錯誤使用 `Result` 類型自然地在管道中流動：

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
            Ok(n) => format!("結果: {}", n),
            Err(e) => format!("錯誤: {}", e),
        }
    }
]?;
```

### 類型轉換

當階段返回 `Result` 時，下一階段接收該 `Result`：

```rust
|req: String| -> anyhow::Result<usize> { ... }  // 返回 Result
|req: anyhow::Result<usize>| -> usize {         // 接收 Result
    req.unwrap()
}
```

## 重試語義

Ichika 為可能瞬態失敗的操作提供自動重試。

### 基本重試

使用 `retry` 函數重試操作：

```rust
use ichika::retry;

let result = retry(|| {
    // 可能失敗的操作
    Ok::<_, anyhow::Error>(42)
})?;
```

### 使用策略重試

使用 `RetryPolicy` 控制重試行為：

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // 具有自定義重試策略的操作
    Ok::<_, anyhow::Error>(42)
})?;
```

### RetryPolicy 選項

```rust
pub struct RetryPolicy {
    /// 最大重試次數
    pub max_attempts: usize,

    /// 初始退避持續時間（應用指數退避）
    pub backoff: Duration,

    /// 最大退避持續時間
    pub max_backoff: Duration,

    /// 是否在退避計算中使用抖動
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

## 在管道中使用重試

### 在階段內重試

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // 重試獲取操作
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // 可能失敗的模擬獲取
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("網絡錯誤"))
                } else {
                    Ok(format!("已獲取: {}", req))
                }
            }
        )
    }
]?;
```

### 在管道級別重試

為了更多控制，在調用者級別處理重試：

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
                None => Err(anyhow::anyhow!("管道已終止")),
            }
        }
    )
}
```

## 錯誤恢復策略

### 回退值

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  // 出錯時默認為 0
    }
]?;
```

### 錯誤聚合

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

### 斷路器模式

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("斷路器已打開"));
        }
        // 處理請求
        Ok(format!("已處理: {}", req))
    }
]?;
```

## 完整示例

這是一個展示錯誤處理和重試的綜合示例：

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("無效輸入: {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // 模擬瞬態故障
            if n % 3 == 0 {
                Err(anyhow::anyhow!("瞬態錯誤"))
            } else {
                Ok(n * 2)
            }
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("成功: {}", n),
                Err(e) => format!("失敗: {}", e),
            }
        }
    ]?;

    // 發送各種輸入
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // 收集結果
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## 最佳實踐

1. **使用 `anyhow::Result`** 進行靈活的錯誤處理
2. **設置適當的重試限制** 以避免無限循環
3. **對網絡操作使用指數退避**
4. **適當記錄錯誤** 以便調試
5. **考慮外部服務調用的斷路器**
6. **使錯誤信息豐富** - 包含有關失敗內容的信息
