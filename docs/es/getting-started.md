# Inicio rápido

Esta guía te ayudará a comenzar con Ichika, desde la instalación hasta tu primera canalización.

## Instalación

Agrega Ichika a tu `Cargo.toml`:

```toml
[dependencies]
ichika = "0.1"
```

### Características

Ichika soporta diferentes tiempos de ejecución asíncronos a través de características:

```toml
# Para tokio (predeterminado)
ichika = { version = "0.1", features = ["tokio"] }

# Para async-std
ichika = { version = "0.1", features = ["async-std"] }

# Para ambos tiempos de ejecución
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## Tu primera canalización

Creemos una canalización simple que procesa cadenas:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // Definir una canalización de 3 etapas
    let pool = pipe![
        // Etapa 1: Analizar cadena a número
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("Fallo de análisis: {}", e))
        },
        // Etapa 2: Duplicar el número
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // Etapa 3: Convertir de vuelta a cadena
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("Error: {}", e))
        }
    ]?;

    // Procesar datos
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // Recopilar resultados
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("Resultado: {}", result);
        }
    }

    Ok(())
}
```

## Entendiendo lo básico

### La macro pipe!

La macro `pipe!` crea una secuencia de etapas de procesamiento. Cada etapa:

1. Recibe entrada de la etapa anterior (o de la llamada inicial `send()`)
2. Procesa los datos en un grupo de hilos
3. Pasa el resultado a la siguiente etapa

### Inferencia de tipos

Ichika infiere automáticamente los tipos que fluyen a través de la canalización:

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### Manejo de errores

Cada etapa puede devolver un `Result`, y los errores se propagan automáticamente:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // o manejar el error apropiadamente
    }
]?;
```

## Próximos pasos

- Aprende más sobre la [macro pipe!](./pipe-macro.md)
- Entiende el [rasgo ThreadPool](./threadpool-trait.md)
- Explora el [manejo de errores](./error-handling.md) en profundidad
- Ve más [ejemplos](./examples.md)
