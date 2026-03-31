# ThreadPool 트레이트

`ThreadPool` 트레이트는 `pipe!` 매크로로 생성된 모든 파이프라인 풀의 인터페이스를 정의합니다.

## 트레이트 정의

```rust
pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&self, req: Self::Request) -> Result<()>;
    fn recv(&self) -> Result<Option<Self::Response>>;

    fn thread_usage(&self) -> Result<usize>;
    fn task_count(&self, id: impl ToString) -> Result<usize>;
}
```

## 메서드

### send

처리를 위해 파이프라인에 요청을 보냅니다.

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**매개변수:**
- `req` - 보낼 요청, 파이프라인의 입력 유형과 일치해야 함

**반환:**
- `Result<()>` - 성공적으로 대기열에 들어가면 Ok, 전송 실패 시 Err

**예제:**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

파이프라인에서 다음 처리된 결과를 수신합니다.

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**반환:**
- `Ok(Some(response))` - 처리된 결과
- `Ok(None)` - 파이프라인이 종료됨
- `Err(...)` - 수신 중 오류 발생

**예제:**

```rust
loop {
    match pool.recv()? {
        Some(result) => println!("받음: {}", result),
        None => break,
    }
}
```

### thread_usage

파이프라인에서 현재 사용 중인 스레드 수를 반환합니다.

```rust
fn thread_usage(&self) -> Result<usize>
```

**반환:**
- 모든 단계의 활성 스레드 총수

**예제:**

```rust
println!("활성 스레드: {}", pool.thread_usage()?);
```

### task_count

명명된 단계의 보류 중인 작업 수를 반환합니다.

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**매개변수:**
- `id` - 단계 이름(`#[name(...)]` 속성으로 설정)

**반환:**
- 해당 단계 큐에서 대기 중인 작업 수

**예제:**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("파서 큐 깊이: {}", pool.task_count("parser")?);
```

## 유형 매개변수

### Request

파이프라인의 입력 유형. 이는 첫 번째 단계가 받아들이는 유형입니다.

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

파이프라인의 출력 유형. 이는 마지막 단계가 반환하는 유형입니다.

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## 수명 주기

파이프라인은 다음 수명 주기를 따릅니다:

1. **생성됨** - `pipe!` 매크로가 새 풀을 반환
2. **활성** - `send()` 요청하고 `recv()` 결과 가능
3. **비우는 중** - 삭제되면 풀은 보류 중인 작업 처리 완료
4. **종료됨** - 풀이 종료되면 `recv()`가 `None`을 반환

## 우아한 종료

풀이 삭제되면 다음을 수행합니다:

1. 새 요청 수신 중단
2. 대기열에 있는 모든 작업 처리 완료
3. 모든 스레드 풀을 우아하게 종료

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // 풀이 범위를 벗어나고 우아하게 종료됨
}
```

## 모니터링

모니터링 메서드를 사용하여 파이프라인 상태 추적:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("stage1")]
        |req: String| -> usize { req.len() },
        #[name("stage2")]
        |req: usize| -> String { req.to_string() }
    ]?;

    // 작업 전송
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // 진행 상황 모니터링
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "스레드: {}, 단계1 보류: {}, 단계2 보류: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
```
