# 高度な機能

このセクションでは、Ichikaを最大限に活用するための高度な機能とテクニックについて説明します。

## 非同期統合

Ichika は `tokio` と `async-std` ランタイムをサポートしています。機能フラグで有効にします：

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# または
ichika = { version = "0.1", features = ["async-std"] }
```

### 非同期ステージ

同期ステージと非同期ステージをシームレスに混ぜることができます：

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // 同期ステージ
        },
        async |req: usize| -> String {
            // 非同期ステージ - tokioランタイムで実行
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## カスタムスレッドクリエイター

各ステージでスレッドをどのように作成するかをカスタマイズできます：

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // 2MBスタック
            .spawn(|| {
                // カスタムスレッドロジック
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## 監視と可観測性

### スレッド使用状況の追跡

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// 総スレッド数を取得
let total_threads = pool.thread_usage()?;

// 名前付きステージの保留中タスクを取得
let pending = pool.task_count("worker")?;

println!("スレッド: {}, 保留中: {}", total_threads, pending);
```

### ヘルスチェック

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## リソース管理

### グレースフルシャットダウン

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// 監視スレッドを生成
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // プールの健全性を監視
        thread::sleep(Duration::from_secs(1));
    }
});

// 完了したら、runningをfalseに設定
running.store(false, Ordering::Relaxed);
// ドロップされるとプールはグレースフルにシャットダウンします
```

### メモリの考慮事項

各ステージには境界付きキューがあります。メモリ制約に応じてキューサイズを調整します：

```rust
let pool = pipe![
    #[queue(100)]   // メモリ制限環境向けの小さいキュー
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  // 高スループットステージ向けの大きいキュー
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## パイプラインパターン

### ファンアウト/ファンイン

項目を並列処理して結果を収集します：

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<String> {
        req.into_iter()
            .filter(|s| !s.is_empty())
            .collect()
    },
    |req: Vec<String>| -> usize {
        req.len()
    }
]?;
```

### 状態を持つ処理

`Arc<Mutex<T>>` を使用して状態を持つステージを作ります：

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("処理済みアイテム数: {}", *count);
        req.len()
    }
]?;
```

### 条件付きルーティング

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("ログイン: {}", user),
            Event::Logout(user) => format!("ログアウト: {}", user),
            Event::Message(from, msg) => format!("{}: {}", from, msg),
        }
    }
]?;
```

## パフォーマンスチューニング

### スレッドプールサイズの調整

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  // CPU数に一致
    |req: String| -> usize {
        req.len()
    }
]?;
```

### バッチ処理

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  // rayonを使用した並列処理
            .map(|s| s.len())
            .collect()
    }
]?;
```

## パイプラインのテスト

### ステージのユニットテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline() {
        let pool = pipe![
            |req: String| -> usize { req.len() },
            |req: usize| -> String { req.to_string() }
        ].unwrap();

        pool.send("test".to_string()).unwrap();
        let result = pool.recv().unwrap().unwrap();
        assert_eq!(result, "4");
    }
}
```

### 統合テスト

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
        }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // パイプラインはエラーを適切に処理するはずです
}
```

## ベストプラクティス

1. **ステージに名前を付ける** - 監視とデバッグのため
2. **適切なスレッド数を使用** - CPUを過剰にサブスクライブしない
3. **合理的なキューサイズを設定** - メモリ使用を制限するため
4. **エラーを明示的に処理** - 失敗を黙って無視しない
5. **本番環境でリソース使用を監視**
6. **エラーパスをテスト** - ハッピーパスだけではない
7. **バックプレッシャを考慮** - ダウンストリームが遅い場合どうなるか？
8. **I/Oバウンドステージには非同期を**、CPUバウンドには同期を使用
