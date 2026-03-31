# 例

このページには、様々な Ichika 機能を示す実用的な例が含まれています。

## 目次

- [基本同期パイプライン](#基本同期パイプライン)
- [基本非同期パイプライン](#基本非同期パイプライン)
- [エラー処理](#エラー処理)
- [グレースフルシャットダウン](#グレースフルシャットダウン)
- [スレッド使用状況の監視](#スレッド使用状況の監視)
- [タプルペイロードパイプライン](#タプルペイロードパイプライン)

## 基本同期パイプライン

シンプルな2ステージ同期パイプラインを示す最小限の例：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("'{}' を長さに変換", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("長さ {} を文字列に戻す", req);
            Ok(req.to_string())
        }
    ]?;

    let inputs = vec!["hello", "world", "ichika"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    loop {
        match pool.recv()? {
            Some(output) => log::info!("受信: {}", output),
            None => break,
        }
    }

    Ok(())
}
```

## 基本非同期パイプライン

tokio を使用した非同期ステージの例：

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("ステージ1: {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("ステージ2: 処理 {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("結果: {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## エラー処理

パイプラインを通じたエラー伝播を示します：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("parse")]
        |req: String| -> anyhow::Result<i32> {
            log::info!("パース: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("処理: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("結果: {}", n),
                Err(e) => format!("エラー: {}", e),
            }
        }
    ]?;

    let inputs = vec!["42", "100", "invalid", "200"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    std::thread::sleep(std::time::Duration::from_millis(100));

    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## グレースフルシャットダウン

パイプラインがドロップされるときの適切なクリーンアップを示します：

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("処理: {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // ワークを送信
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // 処理のための時間を与える
        std::thread::sleep(Duration::from_millis(200));

        // プールはドロップされ、グレースフルにシャットダウンします
        log::info!("プールがスコープ外になります...");
    }

    log::info!("プールはグレースフルにシャットダウンしました");

    Ok(())
}
```

## スレッド使用状況の監視

スレッド使用とタスクカウントを追跡します：

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("stage1")]
        |req: String| -> usize {
            std::thread::sleep(Duration::from_millis(100));
            req.len()
        },
        #[name("stage2")]
        |req: usize| -> String {
            req.to_string()
        }
    ]?;

    // ワークを送信
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // 進行状況を監視
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "スレッド: {}, ステージ1: {}, ステージ2: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("すべてのタスクが完了しました");

    Ok(())
}
```

## タプルペイロードパイプライン

タプルペイロードを処理します：

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}' の長さは {} です", req.0, req.1)
        }
    ]?;

    pool.send("hello".to_string())?;
    pool.send("world".to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## 例の実行

すべての例はリポジトリで利用可能です：

```bash
# 特定の例を実行
cargo run --example basic_sync_chain

# ログ記録付きで実行
RUST_LOG=info cargo run --example basic_sync_chain

# 非同期例を実行
cargo run --example basic_async_chain --features tokio
```

## その他の例

より多くの完全な例については、リポジトリの `examples/` ディレクトリを参照してください：

- `basic_sync_chain.rs` - 同期パイプライン
- `basic_async_chain.rs` - 非同期パイプライン
- `error_handling.rs` - エラー伝播
- `graceful_shutdown_drop.rs` - ドロップ時のクリーンアップ
- `monitoring_thread_usage.rs` - 監視API
- `tuple_payload_pipeline.rs` - 複雑なペイロード型
- `status_exit_demo.rs` - ステータスと終了処理
