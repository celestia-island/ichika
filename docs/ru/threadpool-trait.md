# Трейт ThreadPool

Трейт `ThreadPool` определяет интерфейс для всех пулов конвейеров, созданных макросом `pipe!`.

## Определение трейта

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

## Методы

### send

Отправляет запрос в конвейер для обработки.

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**Параметры:**
- `req` — Запрос для отправки, должен соответствовать типу ввода конвейера

**Возвращает:**
- `Result<()>` — Ok при успешной постановке в очередь, Err при ошибке отправки

**Пример:**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

Получает следующий обработанный результат из конвейера.

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**Возвращает:**
- `Ok(Some(response))` — Обработанный результат
- `Ok(None)` — Конвейер завершил работу
- `Err(...)` — Произошла ошибка при получении

**Пример:**

```rust
loop {
    match pool.recv()? {
        Some(result) => println!("Получено: {}", result),
        None => break,
    }
}
```

### thread_usage

Возвращает текущее количество потоков, используемых конвейером.

```rust
fn thread_usage(&self) -> Result<usize>
```

**Возвращает:**
- Общее количество активных потоков на всех этапах

**Пример:**

```rust
println!("Активные потоки: {}", pool.thread_usage()?);
```

### task_count

Возвращает количество ожидающих задач для именованного этапа.

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**Параметры:**
- `id` — Имя этапа (установленное атрибутом `#[name(...)]`)

**Возвращает:**
- Количество задач, ожидающих в очереди этого этапа

**Пример:**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("Глубина очереди parser: {}", pool.task_count("parser")?);
```

## Параметры типа

### Request

Тип ввода конвейера. Это тип, принимаемый первым этапом.

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

Тип вывода конвейера. Это тип, возвращаемый последним этапом.

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## Жизненный цикл

Конвейер следует этому жизненному циклу:

1. **Создан** — Макрос `pipe!` возвращает новый пул
2. **Активен** — Вы можете вызывать `send()` для запросов и `recv()` для результатов
3. **Опорожнение** — При удалении пул завершает обработку ожидающих задач
4. **Завершён** — `recv()` возвращает `None` когда пул завершает работу

## Корректное завершение

При удалении пула он:

1. Перестаёт принимать новые запросы
2. Завершает обработку всех задач в очереди
3. Корректно завершает работу всех пулов потоков

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // пул выходит из области видимости и завершает работу корректно
}
```

## Мониторинг

Используйте методы мониторинга для отслеживания состояния конвейера:

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

    // Отправляем работу
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // Мониторим прогресс
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "Потоки: {}, Stage1 ожидают: {}, Stage2 ожидают: {}",
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
