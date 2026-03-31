# Макрос pipe!

Макрос `pipe!` — это ядро Ichika. Он преобразует последовательность замыканий в полнофункциональный многоэтапный конвейер обработки.

## Базовый синтаксис

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... больше замыканий
]?;
```

Каждое замыкание представляет один этап обработки в вашем конвейере.

### Сигнатуры замыканий

Каждое замыкание должно следовать этим правилам:

1. **Принимать ровно один параметр** — ввод с предыдущего этапа
2. **Возвращать тип** — который становится вводом для следующего этапа
3. Быть `Clone + Send + 'static` — требуется для выполнения в пуле потоков

### Примеры сигнатур

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // Обрабатываем Result
}
```

## Вывод типов

Ichika автоматически соединяет тип вывода одного этапа с типом ввода следующего:

```rust
let pool = pipe![
    |req: String| -> usize {        // Этап 1: String -> usize
        req.len()
    },
    |req: usize| -> String {         // Этап 2: usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // Этап 3: String -> bool
        !req.is_empty()
    }
]?;
```

## Атрибуты этапов

Вы можете настроить отдельные этапы, используя атрибуты:

### Настройка пула потоков

```rust
let pool = pipe![
    #[threads(4)]                    // Используем 4 потока для этого этапа
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // Используем 2 потока для этого этапа
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### Настройка очереди

```rust
let pool = pipe![
    #[queue(100)]                    // Ёмкость очереди 100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### Именованные этапы

```rust
let pool = pipe![
    #[name("parser")]                # Именуем этап для мониторинга
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

# Запрашиваем количество задач для именованного этапа
let count = pool.task_count("parser")?;
```

## Разветвлённые конвейеры

Вы можете создать условное ветвление в вашем конвейере:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // Обрабатываем каждую ветку
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("Число: {}", n),
            Either::Right(s) => format!("Строка: {}", s),
        }
    }
]?;
```

## Асинхронные этапы

С соответствующими функциями вы можете использовать асинхронные этапы:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // Выполняется в асинхронном окружении
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## Глобальные ограничения

Вы можете установить глобальные ограничения для всего конвейера:

```rust
let pool = pipe![
    #[global_threads(8)]             # Количество потоков по умолчанию для всех этапов
    #[global_queue(1000)]            # Ёмкость очереди по умолчанию
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## Полный пример

Вот более реалистичный пример, показывающий несколько функций:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        #[name("parse")]
        #[threads(2)]
        |req: String| -> anyhow::Result<i32> {
            log::info!("Парсинг: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        #[threads(4)]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("Обработка: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("Форматирование: {}", n);
                    format!("Результат: {}", n)
                }
                Err(e) => {
                    log::error!("Ошибка: {}", e);
                    format!("Ошибка: {}", e)
                }
            }
        }
    ]?;

    // Мониторинг использования потоков
    println!("Использование потоков: {}", pool.thread_usage()?);

    Ok(())
}
```
