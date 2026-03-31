# エラー処理とリトライ

Ichika は強力なエラー処理を提供し、一時的な障害を処理するための組み込みリトライセマンティクスがあります。

## エラー伝播

エラーは `Result` 型を使用してパイプラインを自然に流れます：

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
            Err(e) => format!("エラー: {}", e),
        }
    }
]?;
```

### 型変換

ステージが `Result` を返すと、次のステージはその `Result` を受け取ります：

```rust
|req: String| -> anyhow::Result<usize> { ... }  // Result を返す
|req: anyhow::Result<usize>| -> usize {         // Result を受け取る
    req.unwrap()
}
```

## リトライセマンティクス

Ichika は一時的に失敗する可能性がある操作の自動リトライを提供します。

### 基本リトライ

`retry` 関数を使用して操作をリトライします：

```rust
use ichika::retry;

let result = retry(|| {
    // 失敗する可能性のある操作
    Ok::<_, anyhow::Error>(42)
})?;
```

### ポリシーを使用したリトライ

`RetryPolicy` を使用してリトライ動作を制御します：

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // カスタムリトライポリシーを持つ操作
    Ok::<_, anyhow::Error>(42)
})?;
```

### RetryPolicy オプション

```rust
pub struct RetryPolicy {
    /// 最大リトライ回数
    pub max_attempts: usize,

    /// 初期バックオフ継続時間（指数バックオフが適用されます）
    pub backoff: Duration,

    /// 最大バックオフ継続時間
    pub max_backoff: Duration,

    /// バックオフ計算でジッターを使用するかどうか
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

## パイプラインでのリトライ使用

### ステージ内リトライ

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // 取得操作をリトライ
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // 失敗する可能性のある模擬取得
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("ネットワークエラー"))
                } else {
                    Ok(format!("取得済み: {}", req))
                }
            }
        )
    }
]?;
```

### パイプラインレベルリトライ

より多くの制御のために、呼び出し元レベルでリトライを処理します：

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
                None => Err(anyhow::anyhow!("パイプラインが終了しました")),
            }
        }
    )
}
```

## エラー回復戦略

### フォールバック値

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  // エラー時にデフォルトで0
    }
]?;
```

### エラー集約

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

### サーキットブレーカーパターン

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("サーキットブレーカーが開いています"));
        }
        // リクエストを処理
        Ok(format!("処理済み: {}", req))
    }
]?;
```

## 完全な例

エラー処理とリトライを示す包括的な例：

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("無効な入力: {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // 一時的な障害をシミュレート
            if n % 3 == 0 {
                Err(anyhow::anyhow!("一時的なエラー"))
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

    // 様々な入力を送信
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // 結果を収集
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## ベストプラクティス

1. **`anyhow::Result` を使用** して柔軟なエラー処理を行う
2. **適切なリトライ制限を設定** して無限ループを避ける
3. **ネットワーク操作には指数バックオフを使用**
4. **エラーを適切にログ記録** してデバッグを容易にする
5. **外部サービス呼び出しのサーキットブレーカーを検討**
6. **エラー情報を有用に** - 何が失敗したかに関するコンテキストを含める
