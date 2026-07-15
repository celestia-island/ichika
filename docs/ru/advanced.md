# Расширенные возможности

Этот раздел охватывает расширенные функции и методы для максимального использования Ichika.

## Асинхронная интеграция

Ichika поддерживает окружения `tokio` и `async-std`. Включите с помощью функций:

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# или
ichika = { version = "0.1", features = ["async-std"] }
```

### Асинхронные этапы

Смешивайте синхронные и асинхронные этапы без проблем:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // Синхронный этап
        },
        async |req: usize| -> String {
            // Асинхронный этап — выполняется в окружении tokio
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## Пользовательские создатели потоков

Вы можете настроить создание потоков для каждого этапа:

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // Стек 2МБ
            .spawn(|| {
                // Пользовательская логика потока
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## Мониторинг и наблюдаемость

### Отслеживание использования потоков

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// Получаем общее количество потоков
let total_threads = pool.thread_usage()?;

// Получаем количество ожидающих задач для именованного этапа
let pending = pool.task_count("worker")?;

println!("Потоки: {}, Ожидают: {}", total_threads, pending);
```

### Проверки работоспособности

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## Управление ресурсами

### Корректное завершение

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// Запускаем поток мониторинга
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // Мониторим состояние пула
        thread::sleep(Duration::from_secs(1));
    }
});

// При завершении устанавливаем running в false
running.store(false, Ordering::Relaxed);
// Пул завершит работу корректно при удалении
```

### Соображения по памяти

У каждого этапа есть ограниченная очередь. Настраивайте размеры очередей в соответствии с ограничениями памяти:

```rust
let pool = pipe![
    #[queue(100)]   # Маленькая очередь для сред с ограничениями памяти
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  # Большая очередь для высокопроизводительных этапов
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## Паттерны конвейеров

### Fan-out / Fan-in

Обрабатывайте элементы параллельно и собирайте результаты:

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

### Обработка с состоянием

Используйте `Arc<Mutex<T>>` для этапов с состоянием:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("Обработано элементов: {}", *count);
        req.len()
    }
]?;
```

### Условная маршрутизация

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("Вход: {}", user),
            Event::Logout(user) => format!("Выход: {}", user),
            Event::Message(from, msg) => format!("{}: {}", from, msg),
        }
    }
]?;
```

## Настройка производительности

### Настройка размера пула потоков

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  # Соответствует количеству CPU
    |req: String| -> usize {
        req.len()
    }
]?;
```

### Пакетная обработка

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  # Используем rayon для параллельной обработки
            .map(|s| s.len())
            .collect()
    }
]?;
```

## Тестирование конвейеров

### Модульное тестирование этапов

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

### Интеграционное тестирование

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
    }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // Конвейер должен обрабатывать ошибки корректно
}
```

## Лучшие практики

1. **Называйте ваши этапы** для лучшего мониторинга и отладки
2. **Используйте соответствующие количества потоков** — не перегружайте CPU
3. **Устанавливайте разумные размеры очередей** для ограничения использования памяти
4. **Обрабатывайте ошибки явно** — не игнорируйте сбои молчаливо
5. **Мониторьте использование ресурсов** в продакшене
6. **Тестируйте пути ошибок** — не только счастливые пути
7. **Рассмотрите обратное давление** — что происходит, когда downstream медленный?
8. **Используйте async для I/O-bound этапов**, sync для CPU-bound
