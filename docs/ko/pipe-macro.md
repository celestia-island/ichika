# pipe! 매크로

`pipe!` 매크로는 Ichika의 핵심입니다. 일련의 클로저를 완전한 기능의 다단계 처리 파이프라인으로 변환합니다.

## 기본 구문

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... 더 많은 클로저
]?;
```

각 클로저는 파이프라인의 하나의 처리 단계를 나타냅니다.

## 클로저 서명

각 클로저는 다음 규칙을 따라야 합니다:

1. **정확히 하나의 매개변수 받기** - 이전 단계의 입력
2. **유형 반환** - 이는 다음 단계의 입력이 됨
3. `Clone + Send + 'static` - 스레드 풀 실행에 필요

### 서명 예제

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // Result 처리
}
```

## 유형 유추

Ichika는 한 단계의 출력 유형을 다음 단계의 입력 유형에 자동으로 연결합니다:

```rust
let pool = pipe![
    |req: String| -> usize {        // 단계 1: String -> usize
        req.len()
    },
    |req: usize| -> String {         // 단계 2: usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // 단계 3: String -> bool
        !req.is_empty()
    }
]?;
```

## 단계 속성

속성을 사용하여 개별 단계를 구성할 수 있습니다:

### 스레드 풀 구성

```rust
let pool = pipe![
    #[threads(4)]                    // 이 단계는 4개 스레드 사용
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // 이 단계는 2개 스레드 사용
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### 큐 구성

```rust
let pool = pipe![
    #[queue(100)]                    // 큐 용량 100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### 명명된 단계

```rust
let pool = pipe![
    #[name("parser")]                // 모니터링을 위해 단계 이름 지정
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

// 명명된 단계의 작업 수 쿼리
let count = pool.task_count("parser")?;
```

## 분기 파이프라인

파이프라인 내에서 조건부 분기를 만들 수 있습니다:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // 각 분기 처리
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("숫자: {}", n),
            Either::Right(s) => format!("문자열: {}", s),
        }
    }
]?;
```

## 비동기 단계

적절한 기능 플래그를 사용하면 비동기 단계를 사용할 수 있습니다:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // 비동기 런타임에서 실행
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## 전역 제약 조건

전체 파이프라인에 전역 제약 조건을 설정할 수 있습니다:

```rust
let pool = pipe![
    #[global_threads(8)]             // 모든 단계의 기본 스레드 수
    #[global_queue(1000)]            // 기본 큐 용량
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## 완전한 예제

여러 기능을 보여주는 더 실용적인 예제:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        #[name("parse")]
        #[threads(2)]
        |req: String| -> anyhow::Result<i32> {
            log::info!("구문 분석: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        #[threads(4)]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("처리: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("형식화: {}", n);
                    format!("결과: {}", n)
                }
                Err(e) => {
                    log::error!("오류: {}", e);
                    format!("오류: {}", e)
                }
            }
        }
    ]?;

    // 스레드 사용 현황 모니터링
    println!("스레드 사용 현황: {}", pool.thread_usage()?);

    Ok(())
}
```
