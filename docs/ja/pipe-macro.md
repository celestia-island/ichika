# pipe! マクロ

`pipe!` マクロは Ichika の核心です。一連のクロージャを完全に機能するマルチステージ処理パイプラインに変換します。

## 基本構文

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... その他のクロージャ
]?;
```

各クロージャはパイプライン内の1つの処理ステージを表します。

## クロージャのシグネチャ

各クロージャは以下のルールに従う必要があります：

1. **ちょうど1つのパラメータを受け取る** - 前のステージからの入力
2. **型を返す** - これは次のステージへの入力になります
3. `Clone + Send + 'static` であること - スレッドプール実行に必要

### シグネチャの例

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // Result を処理
}
```

## 型推論

Ichika はあるステージの出力型を次のステージの入力型に自動的に接続します：

```rust
let pool = pipe![
    |req: String| -> usize {        // ステージ1: String -> usize
        req.len()
    },
    |req: usize| -> String {         // ステージ2: usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // ステージ3: String -> bool
        !req.is_empty()
    }
]?;
```

## ステージ属性

属性を使用して個々のステージを設定できます：

### スレッドプール設定

```rust
let pool = pipe![
    #[threads(4)]                    // このステージは4つのスレッドを使用
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // このステージは2つのスレッドを使用
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### キュー設定

```rust
let pool = pipe![
    #[queue(100)]                    // キュー容量100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### 名前付きステージ

```rust
let pool = pipe![
    #[name("parser")]                // 監視のためにステージに名前を付ける
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

// 名前付きステージのタスク数をクエリ
let count = pool.task_count("parser")?;
```

## 分岐パイプライン

パイプライン内で条件分岐を作成できます：

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // 各分岐を処理
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("数値: {}", n),
            Either::Right(s) => format!("文字列: {}", s),
        }
    }
]?;
```

## 非同期ステージ

適切な機能フラグを有効にすると、非同期ステージを使用できます：

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // 非同期ランタイムで実行
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## グローバル制約

パイプライン全体にグローバル制約を設定できます：

```rust
let pool = pipe![
    #[global_threads(8)]             // すべてのステージのデフォルトスレッド数
    #[global_queue(1000)]            // デフォルトキュー容量
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## 完全な例

複数の機能を示すより実践的な例：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        #[name("parse")]
        #[threads(2)]
        |req: String| -> anyhow::Result<i32> {
            log::info!("パース: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        #[threads(4)]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("処理: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("フォーマット: {}", n);
                    format!("結果: {}", n)
                }
                Err(e) => {
                    log::error!("エラー: {}", e);
                    format!("エラー: {}", e)
                }
            }
        }
    ]?;

    // スレッド使用状況を監視
    println!("スレッド使用状況: {}", pool.thread_usage()?);

    Ok(())
}
```
