# Ejemplos

Esta página contiene ejemplos prácticos que demuestran varias características de Ichika.

## Tabla de contenidos

- [Canalización sincrónica básica](#canalización-sincrónica-básica)
- [Canalización asíncrona básica](#canalización-asíncrona-básica)
- [Manejo de errores](#manejo-de-errores)
- [Apagado elegante](#apagado-elegante)
- [Monitoreo del uso de hilos](#monitoreo-del-uso-de-hilos)
- [Canalización con payload tupla](#canalización-con-payload-tupla)

## Canalización sincrónica básica

Un ejemplo mínimo mostrando una canalización sincrónica simple de 2 etapas:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("Convirtiendo '{}' a longitud", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("Convirtiendo longitud {} de vuelta a cadena", req);
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
            Some(output) => log::info!("Recibido: {}", output),
            None => break,
        }
    }

    Ok(())
}
```

## Canalización asíncrona básica

Ejemplo usando etapas asíncronas con tokio:

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("Etapa 1: {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("Etapa 2: procesando {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("Resultado: {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Manejo de errores

Demostrando propagación de errores a través de la canalización:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("parse")]
        |req: String| -> anyhow::Result<i32> {
            log::info!("Analizando: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("Procesando: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("Resultado: {}", n),
                Err(e) => format!("Error: {}", e),
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

## Apagado elegante

Demostrando limpieza adecuada cuando la canalización se descarta:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("Procesando: {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // Enviar trabajo
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // Dar tiempo para procesamiento
        std::thread::sleep(Duration::from_millis(200));

        // El grupo se descartará y se apagará elegantemente
        log::info!("El grupo sale del alcance...");
    }

    log::info!("El grupo se ha apagado elegantemente");

    Ok(())
}
```

## Monitoreo del uso de hilos

Rastreo del uso de hilos y recuentos de tareas:

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

    // Enviar trabajo
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // Monitorear progreso
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "Hilos: {}, Stage1: {}, Stage2: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("Todas las tareas completadas");

    Ok(())
}
```

## Canalización con payload tupla

Trabajando con payloads tupla:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}' tiene longitud {}", req.0, req.1)
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

## Ejecutar los ejemplos

Todos los ejemplos están disponibles en el repositorio:

```bash
# Ejecutar un ejemplo específico
cargo run --example basic_sync_chain

# Ejecutar con registro
RUST_LOG=info cargo run --example basic_sync_chain

# Ejecutar ejemplo async
cargo run --example basic_async_chain --features tokio
```

## Más ejemplos

Revisa el directorio `examples/` en el repositorio para más ejemplos completos:

- `basic_sync_chain.rs` - Canalización sincrónica
- `basic_async_chain.rs` - Canalización asíncrona
- `error_handling.rs` - Propagación de errores
- `graceful_shutdown_drop.rs` - Limpieza al descartar
- `monitoring_thread_usage.rs` - API de monitoreo
- `tuple_payload_pipeline.rs` - Tipos de payload complejos
- `status_exit_demo.rs` - Manejo de estado y salida
