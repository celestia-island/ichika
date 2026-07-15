# クイックスタート

このガイドでは、インストールから最初のパイプラインまで、Ichika の使用方法を説明します。

## インストール

`Cargo.toml` に Ichika を追加します：

```toml
[dependencies]
ichika = "0.1"
```

### 機能フラグ

機能フラグを使用して、異なる非同期ランタイムをサポートできます：

```toml
# tokio サポート（デフォルト）
ichika = { version = "0.1", features = ["tokio"] }

# async-std サポート
ichika = { version = "0.1", features = ["async-std"] }

# 両方のランタイム
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## 最初のパイプライン

文字列を処理するシンプルなパイプラインを作成しましょう：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // 3ステージパイプラインを定義
    let pool = pipe![
        // ステージ1: 文字列を数値にパース
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("パース失敗: {}", e))
        },
        // ステージ2: 数値を2倍にする
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // ステージ3: 文字列に戻す
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("エラー: {}", e))
        }
    ]?;

    // データを処理
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // 結果を収集
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("結果: {}", result);
        }
    }

    Ok(())
}
```

## 基礎の理解

### pipe! マクロ

`pipe!` マクロは一連の処理ステージを作成します。各ステージ：

1. 前のステージ（または最初の `send()` 呼び出し）から入力を受け取る
2. スレッドプールでデータを処理する
3. 結果を次のステージに渡す

### 型推論

Ichika はパイプラインを流れる型を自動的に推論します：

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### エラー処理

各ステージは `Result` を返すことができ、エラーは自動的に伝播されます：

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // またはエラーを適切に処理
    }
]?;
```

## 次のステップ

- [pipe! マクロ](./pipe-macro.md) の詳細を学ぶ
- [ThreadPool トレイト](./threadpool-trait.md) を理解する
- [エラー処理](./error-handling.md) を深く学ぶ
- 他の [例](./examples.md) を見る
