# ThreadPool Trait

`ThreadPool` trait 定义了由 `pipe!` 宏创建的所有管道池的接口。

## Trait 定义

```rust
pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&self, req: Self::Request) -> Result<()>;
    fn recv(&self) -> Result<Option<Self::Response>>;

    fn thread_usage(&self) -> Result<usize>;
    fn task_count(&self, id: impl ToString) -> Result<usize>;
}
```

## 方法

### send

向管道发送请求以进行处理。

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**参数：**
- `req` - 要发送的请求，必须匹配管道的输入类型

**返回：**
- `Result<()>` - 成功排队时返回 Ok，发送失败时返回 Err

**示例：**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

从管道接收下一个已处理的结果。

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**返回：**
- `Ok(Some(response))` - 已处理的结果
- `Ok(None)` - 管道已终止
- `Err(...)` - 接收时发生错误

**示例：**

```rust
loop {
    match pool.recv()? {
        Some(result) => println!("得到: {}", result),
        None => break,
    }
}
```

### thread_usage

返回管道当前使用的线程数。

```rust
fn thread_usage(&self) -> Result<usize>
```

**返回：**
- 所有阶段中活动线程的总数

**示例：**

```rust
println!("活动线程: {}", pool.thread_usage()?);
```

### task_count

返回命名阶段的待处理任务数。

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**参数：**
- `id` - 阶段名称（由 `#[name(...)]` 属性设置）

**返回：**
- 该阶段队列中等待的任务数

**示例：**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("解析器队列深度: {}", pool.task_count("parser")?);
```

## 类型参数

### Request

管道的输入类型。这是第一个阶段接受的类型。

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

管道的输出类型。这是最后一个阶段返回的类型。

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## 生命周期

管道遵循以下生命周期：

1. **已创建** - `pipe!` 宏返回一个新池
2. **活动** - 您可以 `send()` 请求并 `recv()` 结果
3. **排空** - 当被丢弃时，池完成处理待处理的任务
4. **已终止** - 当池关闭时 `recv()` 返回 `None`

## 优雅关闭

当池被丢弃时，它会：

1. 停止接受新请求
2. 完成处理所有排队的任务
3. 优雅地关闭所有线程池

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // 池超出范围并优雅关闭
}
```

## 监控

使用监控方法跟踪管道健康状况：

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("stage1")]
        |req: String| -> usize { req.len() },
        #[name("stage2")]
        |req: usize| -> String { req.to_string() }
    ]?;

    // 发送工作
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // 监控进度
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "线程: {}, 阶段1 待处理: {}, 阶段2 待处理: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
```
