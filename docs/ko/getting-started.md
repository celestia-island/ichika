# 빠른 시작

이 가이드에서는 설치부터 첫 번째 파이프라인까지 Ichika를 사용하는 방법을 설명합니다.

## 설치

`Cargo.toml`에 Ichika를 추가합니다：

```toml
[dependencies]
ichika = "0.1"
```

### 기능 플래그

기능 플래그를 통해 다양한 비동기 런타임을 지원할 수 있습니다:

```toml
# tokio 지원 (기본값)
ichika = { version = "0.1", features = ["tokio"] }

# async-std 지원
ichika = { version = "0.1", features = ["async-std"] }

# 두 런타임 모두 지원
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## 첫 번째 파이프라인

문자열을 처리하는 간단한 파이프라인을 만들어 보겠습니다:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // 3단계 파이프라인 정의
    let pool = pipe![
        // 단계 1: 문자열을 숫자로 구문 분석
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("구문 분석 실패: {}", e))
        },
        // 단계 2: 숫자를 두 배로
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // 단계 3: 다시 문자열로 변환
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("오류: {}", e))
        }
    ]?;

    // 데이터 처리
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // 결과 수집
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("결과: {}", result);
        }
    }

    Ok(())
}
```

## 기본 이해

### pipe! 매크로

`pipe!` 매크로는 일련의 처리 단계를 만듭니다. 각 단계는:

1. 이전 단계(또는 초기 `send()` 호출)에서 입력을 받음
2. 스레드 풀에서 데이터 처리
3. 다음 단계에 결과 전달

### 유형 유추

Ichika는 파이프라인을 통해 흐르는 유형을 자동으로 유추합니다:

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### 오류 처리

각 단계는 `Result`를 반환할 수 있으며 오류가 자동으로 전파됩니다:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // 또는 오류를 적절하게 처리
    }
]?;
```

## 다음 단계

- [pipe! 매크로](./pipe-macro.md)에 대해 자세히 알아보기
- [ThreadPool 트레이트](./threadpool-trait.md) 이해하기
- [오류 처리](./error-handling.md) 심층 학습
- 더 많은 [예제](./examples.md) 보기
