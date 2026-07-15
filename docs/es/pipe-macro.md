# La macro pipe!

La macro `pipe!` es el núcleo de Ichika. Transforma una secuencia de closures en una canalización de procesamiento de múltiples etapas completamente funcional.

## Sintaxis básica

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... más closures
]?;
```

Cada closure representa una etapa de procesamiento en tu canalización.

## Firmas de closure

Cada closure debe seguir estas reglas:

1. **Aceptar exactamente un parámetro** - la entrada de la etapa anterior
2. **Retornar un tipo** - que se convierte en la entrada de la siguiente etapa
3. Ser `Clone + Send + 'static` - requerido para ejecución del grupo de hilos

### Ejemplos de firmas

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // Manejar el Result
}
```

## Inferencia de tipos

Ichika conecta automáticamente el tipo de salida de una etapa con el tipo de entrada de la siguiente:

```rust
let pool = pipe![
    |req: String| -> usize {        // Etapa 1: String -> usize
        req.len()
    },
    |req: usize| -> String {         // Etapa 2: usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // Etapa 3: String -> bool
        !req.is_empty()
    }
]?;
```

## Atributos de etapa

Puedes configurar etapas individuales usando atributos:

### Configuración del grupo de hilos

```rust
let pool = pipe![
    #[threads(4)]                    // Usar 4 hilos para esta etapa
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // Usar 2 hilos para esta etapa
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### Configuración de cola

```rust
let pool = pipe![
    #[queue(100)]                    // Capacidad de cola de 100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### Etapas nombradas

```rust
let pool = pipe![
    #[name("parser")]                # Nombrar etapa para monitoreo
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

# Consultar recuento de tareas para etapa nombrada
let count = pool.task_count("parser")?;
```

## Canalizaciones con ramificación

Puedes crear ramificación condicional en tu canalización:

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // Manejar cada rama
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("Número: {}", n),
            Either::Right(s) => format!("Cadena: {}", s),
        }
    }
]?;
```

## Etapas asíncronas

Con las características apropiadas, puedes usar etapas asíncronas:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // Se ejecuta en runtime async
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## Restricciones globales

Puedes establecer restricciones globales para toda la canalización:

```rust
let pool = pipe![
    #[global_threads(8)]             # Recuento de hilos predeterminado para todas las etapas
    #[global_queue(1000)]            # Capacidad de cola predeterminada
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## Ejemplo completo

Aquí tienes un ejemplo más realista mostrando múltiples características:

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        #[name("parse")]
        #[threads(2)]
        |req: String| -> anyhow::Result<i32> {
            log::info!("Analizando: {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        #[threads(4)]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("Procesando: {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("Formateando: {}", n);
                    format!("Resultado: {}", n)
                }
                Err(e) => {
                    log::error!("Error: {}", e);
                    format!("Error: {}", e)
                }
            }
        }
    ]?;

    // Monitorear uso de hilos
    println!("Uso de hilos: {}", pool.thread_usage()?);

    Ok(())
}
```
