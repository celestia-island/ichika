# Manejo de errores y reintentos

Ichika proporciona un manejo robusto de errores con semánticas de reintento incorporadas para manejar fallos transitorios.

## Propagación de errores

Los errores fluyen naturalmente a través de la canalización usando tipos `Result`:

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
            Ok(n) => format!("Resultado: {}", n),
            Err(e) => format!("Error: {}", e),
        }
    }
]?;
```

### Transformación de tipo

Cuando una etapa devuelve un `Result`, la siguiente etapa recibe ese `Result`:

```rust
|req: String| -> anyhow::Result<usize> { ... }  # Devuelve Result
|req: anyhow::Result<usize>| -> usize {         # Recibe Result
    req.unwrap()
}
```

## Semánticas de reintento

Ichika proporciona reintento automático para operaciones que pueden fallar transitoriamente.

### Reintento básico

Usa la función `retry` para reintentar una operación:

```rust
use ichika::retry;

let result = retry(|| {
    // Operación que podría fallar
    Ok::<_, anyhow::Error>(42)
})?;
```

### Reintento con política

Controla el comportamiento de reintento con una `RetryPolicy`:

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // Operación con política de reintento personalizada
    Ok::<_, anyhow::Error>(42)
})?;
```

### Opciones de RetryPolicy

```rust
pub struct RetryPolicy {
    /// Número máximo de intentos de reintento
    pub max_attempts: usize,

    /// Duración de retroceso inicial (se aplica retroceso exponencial)
    pub backoff: Duration,

    /// Duración máxima de retroceso
    pub max_backoff: Duration,

    /// Usar o no jitter en el cálculo de retroceso
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

## Usando reintento en canalizaciones

### Reintento dentro de una etapa

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // Reintentar operación de obtención
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // Obtención simulada que podría fallar
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("Error de red"))
                } else {
                    Ok(format!("Obtenido: {}", req))
                }
            }
        )
    }
]?;
```

### Reintento a nivel de canalización

Para más control, maneja el reintento a nivel de llamador:

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
                None => Err(anyhow::anyhow!("La canalización ha terminado")),
            }
        }
    )
}
```

## Estrategias de recuperación de errores

### Valores de reserva

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  # Por defecto a 0 en caso de error
    }
]?;
```

### Agregación de errores

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

### Patrón de disyuntor

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("El disyuntor está abierto"));
        }
        // Procesar solicitud
        Ok(format!("Procesado: {}", req))
    }
]?;
```

## Ejemplo completo

Aquí tienes un ejemplo completo mostrando manejo de errores y reintento:

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("Entrada inválida: {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // Simular fallo transitorio
            if n % 3 == 0 {
                Err(anyhow::anyhow!("Error transitorio"))
            } else {
                Ok(n * 2)
            }
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("Éxito: {}", n),
                Err(e) => format!("Fallo: {}", e),
            }
        }
    ]?;

    // Enviar varias entradas
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // Recopilar resultados
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Mejores prácticas

1. **Usar `anyhow::Result`** para manejo de errores flexible
2. **Establecer límites de reintento apropiados** para evitar bucles infinitos
3. **Usar retroceso exponencial** para operaciones de red
4. **Registrar errores apropiadamente** para depuración
5. **Considerar disyuntores** para llamadas a servicios externos
6. **Hacer los errores informativos** - incluir contexto sobre qué falló
