# Características avanzadas

Esta sección cubre características y técnicas avanzadas para aprovechar al máximo Ichika.

## Integración asíncrona

Ichika soporta los tiempos de ejecución `tokio` y `async-std`. Habilita con características:

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# o
ichika = { version = "0.1", features = ["async-std"] }
```

### Etapas asíncronas

Mezcla etapas sincrónicas y asíncronas sin problemas:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // Etapa sincrónica
        },
        async |req: usize| -> String {
            // Etapa asíncrona - se ejecuta en runtime tokio
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## Creadores de hilos personalizados

Puedes personalizar cómo se crean los hilos para cada etapa:

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // Pila de 2MB
            .spawn(|| {
                // Lógica de hilo personalizada
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## Monitoreo y observabilidad

### Seguimiento del uso de hilos

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// Obtener recuento total de hilos
let total_threads = pool.thread_usage()?;

// Obtener tareas pendientes para etapa nombrada
let pending = pool.task_count("worker")?;

println!("Hilos: {}, Pendiente: {}", total_threads, pending);
```

### Verificaciones de salud

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## Gestión de recursos

### Apagado elegante

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// Generar hilo de monitoreo
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // Monitorear salud del grupo
        thread::sleep(Duration::from_secs(1));
    }
});

// Cuando termine, establecer running a false
running.store(false, Ordering::Relaxed);
// El grupo se apagará elegantemente cuando se descarte
```

### Consideraciones de memoria

Cada etapa tiene una cola delimitada. Ajusta los tamaños de cola según tus restricciones de memoria:

```rust
let pool = pipe![
    #[queue(100)]   # Cola pequeña para entornos con restricciones de memoria
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  # Cola más grande para etapas de alto rendimiento
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## Patrones de canalización

### Fan-out / Fan-in

Procesar elementos en paralelo y recopilar resultados:

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

### Procesamiento con estado

Usa `Arc<Mutex<T>>` para etapas con estado:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("Elementos procesados: {}", *count);
        req.len()
    }
]?;
```

### Enrutamiento condicional

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("Inicio de sesión: {}", user),
            Event::Logout(user) => format!("Cierre de sesión: {}", user),
            Event::Message(from, msg) => format!("{}: {}", from, msg),
        }
    }
]?;
```

## Ajuste de rendimiento

### Ajuste del tamaño del grupo de hilos

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  # Coincide con recuento de CPU
    |req: String| -> usize {
        req.len()
    }
]?;
```

### Procesamiento por lotes

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  # Usar rayon para procesamiento paralelo
            .map(|s| s.len())
            .collect()
    }
]?;
```

## Probar canalizaciones

### Probar etapas unitarias

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

### Pruebas de integración

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
    }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // La canalización debería manejar errores elegantemente
}
```

## Mejores prácticas

1. **Nombrar tus etapas** para mejor monitoreo y depuración
2. **Usar recuentos de hilos apropiados** - no sobrescribir CPU
3. **Establecer tamaños de cola razonables** para limitar uso de memoria
4. **Manejar errores explícitamente** - no ignorar silenciosamente fallos
5. **Monitorear uso de recursos** en producción
6. **Probar caminos de error** - no solo caminos felices
7. **Considerar contrapresión** - qué pasa cuando aguas abajo está lento?
8. **Usar async para etapas I/O-bound**, sync para CPU-bound
