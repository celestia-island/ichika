# 예제

이 페이지는 다양한 Ichika 기능을 보여주는 실용적인 예제를 포함합니다.

## 목차

- [기본 동기 파이프라인](#기본-동기-파이프라인)
- [기본 비동기 파이프라인](#기본-비동기-파이프라인)
- [오류 처리](#오류-처리)
- [우아한 종료](#우아한-종료)
- [스레드 사용 모니터링](#스레드-사용-모니터링)
- [튜플 페이로드 파이프라인](#튜플-페이로드-파이프라인)

## 기본 동기 파이프라인

간단한 2단계 동기 파이프라인을 보여주는 최소한의 예제:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("'{}'를 길이로 변환", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("길이 {}를 다시 문자열로 변환", req);
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
            Some(output) => log::info!("수신: {}", output),
            None => break,
        }
    }

    Ok(())
}
```

## 기본 비동기 파이프라인

tokio를 사용하는 비동기 단계 예제:

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("단계 1: {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("단계 2: 처리 {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("결과: {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## 오류 처리

파이프라인을 통한 오류 전파를 보여줍니다:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("parse")]
        |req: String| -> anyhow::Result<i32> {
            log::info!("구문 분석: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("처리: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("결과: {}", n),
                Err(e) => format!("오류: {}", e),
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

## 우아한 종료

파이프라인이 삭제될 때 적절한 정리를 보여줍니다:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("처리: {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // 작업 전송
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // 처리를 위한 시간 제공
        std::thread::sleep(Duration::from_millis(200));

        // 풀이 삭제되고 우아하게 종료됨
        log::info!("풀이 범위를 벗어남...");
    }

    log::info!("풀이 우아하게 종료됨");

    Ok(())
}
```

## 스레드 사용 모니터링

스레드 사용 및 작업 수 추적:

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

    // 작업 전송
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // 진행 상황 모니터링
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "스레드: {}, 단계1: {}, 단계2: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("모든 작업 완료");

    Ok(())
}
```

## 튜플 페이로드 파이프라인

튜플 페이로드 처리:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}'의 길이는 {}입니다", req.0, req.1)
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

## 예제 실행

모든 예제는 리포지토리에서 제공됩니다:

```bash
# 특정 예제 실행
cargo run --example basic_sync_chain

# 로깅과 함께 실행
RUST_LOG=info cargo run --example basic_sync_chain

# 비동기 예제 실행
cargo run --example basic_async_chain --features tokio
```

## 더 많은 예제

더 많은 완전한 예제는 리포지토리의 `examples/` 디렉터리를 확인하세요:

- `basic_sync_chain.rs` - 동기 파이프라인
- `basic_async_chain.rs` - 비동기 파이프라인
- `error_handling.rs` - 오류 전파
- `graceful_shutdown_drop.rs` - 삭제 시 정리
- `monitoring_thread_usage.rs` - 모니터링 API
- `tuple_payload_pipeline.rs` - 복잡한 페이로드 유형
- `status_exit_demo.rs` - 상태 및 종료 처리
