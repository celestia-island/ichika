# Обработка ошибок и повторы

Ichika обеспечивает надёжную обработку ошибок со встроенными семантиками повтора для обработки временных сбоев.

## Распространение ошибок

Ошибки естественным образом проходят через конвейер, используя типы `Result`:

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
            Ok(n) => format!("Результат: {}", n),
            Err(e) => format!("Ошибка: {}", e),
        }
    }
]?;
```

### Преобразование типа

Когда этап возвращает `Result`, следующий этап получает этот `Result`:

```rust
|req: String| -> anyhow::Result<usize> { ... }  # Возвращает Result
|req: anyhow::Result<usize>| -> usize {         # Получает Result
    req.unwrap()
}
```

## Семантика повтора

Ichika предоставляет автоматический повтор для операций, которые могут временно завершиться неудачей.

### Базовый повтор

Используйте функцию `retry` для повторения операции:

```rust
use ichika::retry;

let result = retry(|| {
    // Операция, которая может завершиться неудачей
    Ok::<_, anyhow::Error>(42)
})?;
```

### Повтор с политикой

Управляйте поведением повтора с помощью `RetryPolicy`:

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // Операция с пользовательской политикой повтора
    Ok::<_, anyhow::Error>(42)
})?;
```

### Опции RetryPolicy

```rust
pub struct RetryPolicy {
    /// Максимальное количество попыток повтора
    pub max_attempts: usize,

    /// Начальная длительность отката (применяется экспоненциальный откат)
    pub backoff: Duration,

    /// Максимальная длительность отката
    pub max_backoff: Duration,

    /// Использовать ли джиттер в расчёте отката
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

## Использование повтора в конвейерах

### Повтор внутри этапа

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // Повторяем операцию получения
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // Имитация получения, которая может завершиться неудачей
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("Ошибка сети"))
                } else {
                    Ok(format!("Получено: {}", req))
                }
            }
        )
    }
]?;
```

### Повтор на уровне конвейера

Для большего контроля обрабатывайте повтор на уровне вызывающего:

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
                None => Err(anyhow::anyhow!("Конвейер завершён")),
            }
        }
    )
}
```

## Стратегии восстановления ошибок

### Значения по умолчанию

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  # По умолчанию 0 при ошибке
    }
]?;
```

### Агрегация ошибок

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

### Паттерн размыкания цепи

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Размыкатель цепи открыт"));
        }
        // Обрабатываем запрос
        Ok(format!("Обработано: {}", req))
    }
]?;
```

## Полный пример

Вот полный пример, показывающий обработку ошибок и повтор:

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("Неверный ввод: {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // Имитация временного сбоя
            if n % 3 == 0 {
                Err(anyhow::anyhow!("Временная ошибка"))
            } else {
                Ok(n * 2)
            }
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("Успех: {}", n),
                Err(e) => format!("Сбой: {}", e),
            }
        }
    ]?;

    // Отправляем несколько вводов
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // Собираем результаты
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Лучшие практики

1. **Используйте `anyhow::Result`** для гибкой обработки ошибок
2. **Устанавливайте подходящие пределы повтора** для избежания бесконечных циклов
3. **Используйте экспоненциальный откат** для сетевых операций
4. **Журналируйте ошибки соответствующим образом** для отладки
5. **Рассмотрите размыкатели цепи** для вызовов внешних сервисов
6. **Делайте ошибки информативными** — включайте контекст о том, что не удалось
