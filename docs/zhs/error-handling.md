# 错误处理与重试

Ichika 提供强大的错误处理，内置重试语义用于处理瞬态故障。

## 错误传播

错误使用 `Result` 类型自然地在管道中流动：

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
            Ok(n) => format!("结果: {}", n),
            Err(e) => format!("错误: {}", e),
        }
    }
]?;
```

### 类型转换

当阶段返回 `Result` 时，下一阶段接收该 `Result`：

```rust
|req: String| -> anyhow::Result<usize> { ... }  // 返回 Result
|req: anyhow::Result<usize>| -> usize {         // 接收 Result
    req.unwrap()
}
```

## 重试语义

Ichika 为可能瞬态失败的操作提供自动重试。

### 基本重试

使用 `retry` 函数重试操作：

```rust
use ichika::retry;

let result = retry(|| {
    // 可能失败的操作
    Ok::<_, anyhow::Error>(42)
})?;
```

### 使用策略重试

使用 `RetryPolicy` 控制重试行为：

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // 具有自定义重试策略的操作
    Ok::<_, anyhow::Error>(42)
})?;
```

### RetryPolicy 选项

```rust
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_attempts: usize,

    /// 初始退避持续时间（应用指数退避）
    pub backoff: Duration,

    /// 最大退避持续时间
    pub max_backoff: Duration,

    /// 是否在退避计算中使用抖动
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

## 在管道中使用重试

### 在阶段内重试

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // 重试获取操作
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // 可能失败的模拟获取
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("网络错误"))
                } else {
                    Ok(format!("已获取: {}", req))
                }
            }
        )
    }
]?;
```

### 在管道级别重试

为了更多控制，在调用者级别处理重试：

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
                None => Err(anyhow::anyhow!("管道已终止")),
            }
        }
    )
}
```

## 错误恢复策略

### 回退值

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  // 出错时默认为 0
    }
]?;
```

### 错误聚合

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

### 断路器模式

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("断路器已打开"));
        }
        // 处理请求
        Ok(format!("已处理: {}", req))
    }
]?;
```

## 完整示例

这是一个展示错误处理和重试的综合示例：

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("无效输入: {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // 模拟瞬态故障
            if n % 3 == 0 {
                Err(anyhow::anyhow!("瞬态错误"))
            } else {
                Ok(n * 2)
            }
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("成功: {}", n),
                Err(e) => format!("失败: {}", e),
            }
        }
    ]?;

    // 发送各种输入
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // 收集结果
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## 最佳实践

1. **使用 `anyhow::Result`** 进行灵活的错误处理
2. **设置适当的重试限制** 以避免无限循环
3. **对网络操作使用指数退避**
4. **适当记录错误** 以便调试
5. **考虑外部服务调用的断路器**
6. **使错误信息丰富** - 包含有关失败内容的信息
