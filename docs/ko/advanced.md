# 고급 기능

이 섹션에서는 Ichika를 최대한 활용하기 위한 고급 기능과 기술을 다룹니다.

## 비동기 통합

Ichika는 `tokio`와 `async-std` 런타임을 지원합니다. 기능 플래그로 활성화:

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# 또는
ichika = { version = "0.1", features = ["async-std"] }
```

### 비동기 단계

동기 및 비동기 단계를 원활하게 혼합:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // 동기 단계
        },
        async |req: usize| -> String {
            // 비동기 단계 - tokio 런타임에서 실행
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## 사용자 지정 스레드 생성자

각 단계에서 스레드 생성 방법을 사용자 지정할 수 있습니다:

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // 2MB 스택
            .spawn(|| {
                // 사용자 지정 스레드 로직
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## 모니터링 및 관찰 가능성

### 스레드 사용 추적

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// 총 스레드 수 가져오기
let total_threads = pool.thread_usage()?;

// 명명된 단계의 보류 중 작업 가져오기
let pending = pool.task_count("worker")?;

println!("스레드: {}, 보류: {}", total_threads, pending);
```

### 상태 점검

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## 리소스 관리

### 우아한 종료

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// 모니터링 스레드 생성
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // 풀 상태 모니터링
        thread::sleep(Duration::from_secs(1));
    }
});

// 완료되면 running을 false로 설정
running.store(false, Ordering::Relaxed);
// 삭제되면 풀이 우아하게 종료됨
```

### 메모리 고려 사항

각 단계는 제한된 큐를 가집니다. 메모리 제약에 따라 큐 크기 조정:

```rust
let pool = pipe![
    #[queue(100)]   // 메모리 제한 환경을 위한 작은 큐
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  # 높은 처리량 단계를 위한 큰 큐
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## 파이프라인 패턴

### 팬아웃/팬인

항목을 병렬로 처리하고 결과 수집:

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

### 상태 저장 처리

`Arc<Mutex<T>>`를 사용하여 상태 저장 단계:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("처리된 항목 수: {}", *count);
        req.len()
    }
]?;
```

### 조건부 라우팅

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("로그인: {}", user),
            Event::Logout(user) => format!("로그아웃: {}", user),
            Event::Message(from, msg) => format!("{}: {}", from, msg),
        }
    }
]?;
```

## 성능 튜닝

### 스레드 풀 크기 조정

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  # CPU 수와 일치
    |req: String| -> usize {
        req.len()
    }
]?;
```

### 배치 처리

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  # rayon을 사용한 병렬 처리
            .map(|s| s.len())
            .collect()
    }
]?;
```

## 파이프라인 테스트

### 단위 테스트 단계

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

### 통합 테스트

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
        }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // 파이프라인은 오류를 우아하게 처리해야 함
}
```

## 모범 사례

1. **단계에 이름 지정** 모니터링 및 디버깅을 위해
2. **적절한 스레드 수 사용** CPU를 과도하게 구독하지 않음
3. **합리적인 큐 크기 설정** 메모리 사용 제한을 위해
4. **오류를 명시적으로 처리** 실패를 조용히 무시하지 않음
5. **프로덕션에서 리소스 사용 모니터링**
6. **오류 경로 테스트** 행복 경로만 테스트하지 않음
7. **백프레저 고려** 다운스트림이 느릴 때 어떻게 되는지?
8. **I/O 바운드 단계에는 비동기 사용**, CPU 바운드에는 동기 사용
