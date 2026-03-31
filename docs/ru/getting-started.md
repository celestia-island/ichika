# Начало работы

Это руководство поможет вам начать работу с Ichika, от установки до вашего первого конвейера.

## Установка

Добавьте Ichika в ваш `Cargo.toml`:

```toml
[dependencies]
ichika = "0.1"
```

### Флаги функций

Ichika поддерживает различные асинхронные окружения через флаги функций:

```toml
# Для поддержки tokio (по умолчанию)
ichika = { version = "0.1", features = ["tokio"] }

# Для поддержки async-std
ichika = { version = "0.1", features = ["async-std"] }

# Для обоих окружений
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## Ваш первый конвейер

Давайте создадим простой конвейер для обработки строк:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // Определяем 3-этапный конвейер
    let pool = pipe![
        // Этап 1: Парсим строку в число
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("Не удалось разобрать: {}", e))
        },
        // Этап 2: Удваиваем число
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // Этап 3: Преобразуем обратно в строку
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("Ошибка: {}", e))
        }
    ]?;

    // Обрабатываем некоторые данные
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // Собираем результаты
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("Результат: {}", result);
        }
    }

    Ok(())
}
```

## Понимание основ

### Макрос pipe!

Макрос `pipe!` создаёт цепочку этапов обработки. Каждый этап:

1. Получает ввод с предыдущего этапа (или из начального вызова `send()`)
2. Обрабатывает данные в пуле потоков
3. Передаёт результат следующему этапу

### Распространение типов

Ichika автоматически выводит типы, проходящие через ваш конвейер:

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### Обработка ошибок

Каждый этап может возвращать `Result`, и ошибки автоматически распространяются:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // или обработайте ошибку соответствующим образом
    }
]?;
```

## Следующие шаги

- Узнайте больше о [макросе pipe!](./pipe-macro.md)
- Поймите [трейт ThreadPool](./threadpool-trait.md)
- Изучите [обработку ошибок](./error-handling.md) подробнее
- Посмотрите больше [примеров](./examples.md)
