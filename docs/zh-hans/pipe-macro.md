# pipe! 宏

`pipe!` 宏是 Ichika 的核心。它将一系列闭包转换为功能齐全的多阶段处理管道。

## 基本语法

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... 更多闭包
]?;
```

每个闭包代表管道中的一个处理阶段。

## 闭包签名

每个闭包必须遵循以下规则：

1. **接受恰好一个参数** - 来自上一阶段的输入
2. **返回一个类型** - 这将成为下一阶段的输入
3. 是 `Clone + Send + 'static` - 线程池执行所需

### 签名示例

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // 处理 Result
}
```

## 类型推断

Ichika 自动将一个阶段的输出类型连接到下一个阶段的输入类型：

```rust
let pool = pipe![
    |req: String| -> usize {        // 阶段 1: String -> usize
        req.len()
    },
    |req: usize| -> String {         // 阶段 2: usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // 阶段 3: String -> bool
        !req.is_empty()
    }
]?;
```

## 阶段属性

您可以使用属性配置各个阶段：

### 线程池配置

```rust
let pool = pipe![
    #[threads(4)]                    // 此阶段使用 4 个线程
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // 此阶段使用 2 个线程
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### 队列配置

```rust
let pool = pipe![
    #[queue(100)]                    // 队列容量为 100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### 命名阶段

```rust
let pool = pipe![
    #[name("parser")]                // 命名阶段以便监控
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

// 查询命名阶段的任务计数
let count = pool.task_count("parser")?;
```

## 分支管道

您可以在管道中创建条件分支：

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // 处理每个分支
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("数字: {}", n),
            Either::Right(s) => format!("字符串: {}", s),
        }
    }
]?;
```

## 异步阶段

启用相应的功能特性后，您可以使用异步阶段：

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // 这在异步运行时中运行
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## 全局约束

您可以为整个管道设置全局约束：

```rust
let pool = pipe![
    #[global_threads(8)]             // 所有阶段的默认线程计数
    #[global_queue(1000)]            // 默认队列容量
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## 完整示例

这是一个展示多个功能的更实际示例：

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
            log::info!("处理: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("格式化: {}", n);
                    format!("结果: {}", n)
                }
                Err(e) => {
                    log::error!("错误: {}", e);
                    format!("错误: {}", e)
                }
            }
        }
    ]?;

    // 监控线程使用情况
    println!("线程使用情况: {}", pool.thread_usage()?);

    Ok(())
}
```
