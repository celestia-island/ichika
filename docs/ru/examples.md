# Примеры

Эта страница содержит практические примеры, демонстрирующие различные возможности Ichika.

## Содержание

- [Базовый синхронный конвейер](#базовый-синхронный-конвейер)
- [Базовый асинхронный конвейер](#базовый-асинхронный-конвейер)
- [Обработка ошибок](#обработка-ошибок)
- [Корректное завершение](#корректное-завершение)
- [Мониторинг использования потоков](#мониторинг-использования-потоков)
- [Конвейер с кортежем в качестве полезной нагрузки](#конвейер-с-кортежем-в-качестве-полезной-нагрузки)

## Базовый синхронный конвейер

Минимальный пример, показывающий простой синхронный конвейер из 2 этапов:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("Преобразование '{}' в длину", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("Преобразование длины {} обратно в строку", req);
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
            Some(output) => log::info!("Получено: {}", output),
            None => break,
        }
    }

    Ok(())
}
```

## Базовый асинхронный конвейер

Пример с использованием асинхронных этапов с tokio:

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("Этап 1: {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("Этап 2: обработка {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("Результат: {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Обработка ошибок

Демонстрация распространения ошибок через конвейер:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("parse")]
        |req: String| -> anyhow::Result<i32> {
            log::info!("Парсинг: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("Обработка: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("Результат: {}", n),
                Err(e) => format!("Ошибка: {}", e),
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

## Корректное завершение

Демонстрация правильной очистки при удалении конвейера:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("Обработка: {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // Отправляем работу
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // Даём время для обработки
        std::thread::sleep(Duration::from_millis(200));

        // Пул будет удалён и завершит работу корректно
        log::info!("Пул выходит из области видимости...");
    }

    log::info!("Пул завершил работу корректно");

    Ok(())
}
```

## Мониторинг использования потоков

Отслеживание использования потоков и количества задач:

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

    // Отправляем работу
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // Мониторим прогресс
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "Потоки: {}, Stage1: {}, Stage2: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("Все задачи завершены");

    Ok(())
}
```

## Конвейер с кортежем в качестве полезной нагрузки

Работа с полезной нагрузкой в виде кортежа:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}' имеет длину {}", req.0, req.1)
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

## Запуск примеров

Все примеры доступны в репозитории:

```bash
# Запустить конкретный пример
cargo run --example basic_sync_chain

# Запустить с журналированием
RUST_LOG=info cargo run --example basic_sync_chain

# Запустить асинхронный пример
cargo run --example basic_async_chain --features tokio
```

## Дополнительные примеры

Ознакомьтесь с директорией `examples/` в репозитории для более полных примеров:

- `basic_sync_chain.rs` — Синхронный конвейер
- `basic_async_chain.rs` — Асинхронный конвейер
- `error_handling.rs` — Распространение ошибок
- `graceful_shutdown_drop.rs` — Очистка при удалении
- `monitoring_thread_usage.rs` — API мониторинга
- `tuple_payload_pipeline.rs` — Сложные типы полезной нагрузки
- `status_exit_demo.rs` — Управление состоянием и выходом
