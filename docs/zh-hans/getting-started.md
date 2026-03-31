# 快速开始

本指南将帮助您开始使用 Ichika，从安装到您的第一个管道。

## 安装

将 Ichika 添加到您的 `Cargo.toml`：

```toml
[dependencies]
ichika = "0.1"
```

### 功能特性

Ichika 通过功能特性支持不同的异步运行时：

```toml
# 对于 tokio 支持（默认）
ichika = { version = "0.1", features = ["tokio"] }

# 对于 async-std 支持
ichika = { version = "0.1", features = ["async-std"] }

# 同时支持两个运行时
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## 您的第一个管道

让我们创建一个处理字符串的简单管道：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // 定义一个 3 阶段管道
    let pool = pipe![
        // 阶段 1: 解析字符串为数字
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("解析失败: {}", e))
        },
        // 阶段 2: 将数字加倍
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // 阶段 3: 转换回字符串
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("错误: {}", e))
        }
    ]?;

    // 处理一些数据
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // 收集结果
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("结果: {}", result);
        }
    }

    Ok(())
}
```

## 理解基础

### pipe! 宏

`pipe!` 宏创建一系列处理阶段。每个阶段：

1. 从上一阶段（或初始的 `send()` 调用）接收输入
2. 在线程池中处理数据
3. 将结果传递给下一阶段

### 类型传播

Ichika 自动推断流经管道的类型：

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### 错误处理

每个阶段可以返回一个 `Result`，错误会自动传播：

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // 或者适当地处理错误
    }
]?;
```

## 下一步

- 了解更多关于 [pipe! 宏](./pipe-macro.md)
- 理解 [ThreadPool trait](./threadpool-trait.md)
- 深入探索 [错误处理](./error-handling.md)
- 查看更多 [示例](./examples.md)
