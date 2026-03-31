# ThreadPool トレイト

`ThreadPool` トレイトは、`pipe!` マクロによって作成されたすべてのパイプラインプールのインターフェースを定義します。

## トレイト定義

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

## メソッド

### send

処理のためにパイプラインにリクエストを送信します。

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**パラメータ:**
- `req` - 送信するリクエスト、パイプラインの入力型と一致する必要があります

**戻り値:**
- `Result<()>` - 正常にキューに入った場合は Ok、送信が失敗した場合は Err

**例:**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

パイプラインから次の処理済み結果を受信します。

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**戻り値:**
- `Ok(Some(response))` - 処理済みの結果
- `Ok(None)` - パイプラインが終了しました
- `Err(...)` - 受信中にエラーが発生しました

**例:**

```rust
loop {
    match pool.recv()? {
        Some(result) => println!("取得: {}", result),
        None => break,
    }
}
```

### thread_usage

パイプラインによって現在使用されているスレッド数を返します。

```rust
fn thread_usage(&self) -> Result<usize>
```

**戻り値:**
- すべてのステージのアクティブなスレッドの合計数

**例:**

```rust
println!("アクティブなスレッド: {}", pool.thread_usage()?);
```

### task_count

名前付きステージの保留中のタスク数を返します。

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**パラメータ:**
- `id` - ステージ名（`#[name(...)]` 属性で設定）

**戻り値:**
- そのステージのキューで待機しているタスク数

**例:**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("パーサーキュー深度: {}", pool.task_count("parser")?);
```

## 型パラメータ

### Request

パイプラインの入力型。これは最初のステージが受け入れる型です。

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

パイプラインの出力型。これは最後のステージが返す型です。

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## ライフサイクル

パイプラインは以下のライフサイクルに従います：

1. **作成済み** - `pipe!` マクロが新しいプールを返す
2. **アクティブ** - `send()` リクエストして `recv()` 結果できる
3. **ドレイン中** - ドロップされると、プールは保留中のタスクの処理を完了します
4. **終了済み** - プールがシャットダウンすると `recv()` が `None` を返す

## グレースフルシャットダウン

プールがドロップされると、以下のようになります：

1. 新しいリクエストの受信を停止します
2. すべてのキューに入ったタスクの処理を完了します
3. すべてのスレッドプールをグレースフルにシャットダウンします

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // プールはスコープ外になり、グレースフルにシャットダウンします
}
```

## 監視

監視メソッドを使用してパイプラインの健全性を追跡します：

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

    // ワークを送信
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // 進行状況を監視
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "スレッド: {}, ステージ1保留中: {}, ステージ2保留中: {}",
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
