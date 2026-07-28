# ThreadPool Trait

`ThreadPool` trait 定義了由 `pipe!` 宏創建的所有管道池的接口。

## Trait 定義

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

向管道發送請求以進行處理。

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**參數：**
- `req` - 要發送的請求，必須匹配管道的輸入類型

**返回：**
- `Result<()>` - 成功排隊時返回 Ok，發送失敗時返回 Err

**示例：**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

從管道接收下一個已處理的結果。

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**返回：**
- `Ok(Some(response))` - 已處理的結果
- `Ok(None)` - 管道已終止
- `Err(...)` - 接收時發生錯誤

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

返回管道當前使用的線程數。

```rust
fn thread_usage(&self) -> Result<usize>
```

**返回：**
- 所有階段中活動線程的總數

**示例：**

```rust
println!("活動線程: {}", pool.thread_usage()?);
```

### task_count

返回命名階段的待處理任務數。

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**參數：**
- `id` - 階段名稱（由 `#[name(...)]` 屬性設置）

**返回：**
- 該階段隊列中等待的任務數

**示例：**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("解析器隊列深度: {}", pool.task_count("parser")?);
```

## 類型參數

### Request

管道的輸入類型。這是第一個階段接受的類型。

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

管道的輸出類型。這是最後一個階段返回的類型。

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## 生命週期

管道遵循以下生命週期：

1. **已創建** - `pipe!` 宏返回一個新池
2. **活動** - 您可以 `send()` 請求並 `recv()` 結果
3. **排空** - 當被丟棄時，池完成處理待處理的任務
4. **已終止** - 當池關閉時 `recv()` 返回 `None`

## 優雅關閉

當池被丟棄時，它會：

1. 停止接受新請求
2. 完成處理所有排隊的任務
3. 優雅地關閉所有線程池

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // 池超出範圍並優雅關閉
}
```

## 監控

使用監控方法跟踪管道健康狀況：

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

    // 發送工作
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // 監控進度
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "線程: {}, 階段1 待處理: {}, 階段2 待處理: {}",
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
