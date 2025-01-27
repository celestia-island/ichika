# 오류 처리 및 재시도

Ichika는 강력한 오류 처리를 제공하며 일시적인 장애 처리를 위한 내장된 재시도 의미 체계가 있습니다.

## 오류 전파

오류는 `Result` 유형을 사용하여 파이프라인을 통해 자연스럽게 흐릅니다:

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
            Ok(n) => format!("결과: {}", n),
            Err(e) => format!("오류: {}", e),
        }
    }
]?;
```

### 유형 변환

단계가 `Result`를 반환하면 다음 단계는 해당 `Result`를 받습니다:

```rust
|req: String| -> anyhow::Result<usize> { ... }  // Result 반환
|req: anyhow::Result<usize>| -> usize {         // Result 수신
    req.unwrap()
}
```

## 재시도 의미 체계

Ichika는 일시적으로 실패할 수 있는 작업에 대한 자동 재시도를 제공합니다.

### 기본 재시도

`retry` 함수를 사용하여 작업을 재시도합니다:

```rust
use ichika::retry;

let result = retry(|| {
    // 실패할 수 있는 작업
    Ok::<_, anyhow::Error>(42)
})?;
```

### 정책을 사용한 재시도

`RetryPolicy`를 사용하여 재시도 동작을 제어합니다:

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // 사용자 지정 재시도 정책이 있는 작업
    Ok::<_, anyhow::Error>(42)
})?;
```

### RetryPolicy 옵션

```rust
pub struct RetryPolicy {
    /// 최대 재시도 횟수
    pub max_attempts: usize,

    /// 초기 백오프 기간(지수 백오프 적용)
    pub backoff: Duration,

    /// 최대 백오프 기간
    pub max_backoff: Duration,

    /// 백오프 계산에 지터 사용 여부
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

## 파이프라인에서 재시도 사용

### 단계 내 재시도

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // 가져오기 작업 재시도
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // 실패할 수 있는 시뮬레이션 가져오기
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("네트워크 오류"))
                } else {
                    Ok(format!("가져옴: {}", req))
                }
            }
        )
    }
]?;
```

### 파이프라인 수준 재시도

더 많은 제어를 위해 호출자 수준에서 재시도를 처리합니다:

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
                None => Err(anyhow::anyhow!("파이프라인이 종료되었습니다")),
            }
        }
    )
}
```

## 오류 복구 전략

### 대체 값

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  // 오류 시 기본값 0
    }
]?;
```

### 오류 집계

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

### 서킷 브레이커 패턴

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("서킷 브레이커가 열려 있습니다"));
        }
        // 요청 처리
        Ok(format!("처리됨: {}", req))
    }
]?;
```

## 완전한 예제

오류 처리와 재시도를 보여주는 포괄적인 예제:

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("잘못된 입력: {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // 일시적인 장애 시뮬레이션
            if n % 3 == 0 {
                Err(anyhow::anyhow!("일시적인 오류"))
            } else {
                Ok(n * 2)
            }
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("성공: {}", n),
                Err(e) => format!("실패: {}", e),
            }
        }
    ]?;

    // 다양한 입력 전송
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // 결과 수집
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## 모범 사례

1. **`anyhow::Result` 사용** 유연한 오류 처리를 위해
2. **적절한 재시도 제한 설정** 무한 루프 방지
3. **네트워크 작업에는 지수 백오프 사용**
4. **오류를 적절하게 로깅** 디버깅 용이성
5. **외부 서비스 호출의 서킷 브레이커 고려**
6. **오류를 유익하게 만들기** - 실패한 내용에 대한 컨텍스트 포함
