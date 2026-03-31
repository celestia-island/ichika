# Rasgo ThreadPool

El rasgo `ThreadPool` define la interfaz para todos los grupos de canalizaciones creados por la macro `pipe!`.

## Definición del rasgo

```rust
pub trait ThreadPool {
    type Request: Clone;
    type Response: Clone;

    fn send(&self, req: Self::Request) -> Result<()>;
    fn recv(&self) -> Result<Option<Self::Response>>;

    fn thread_usage(&self) -> Result<usize>;
    fn task_count(&self, id: impl ToString) -> Result<usize>;
}
```

## Métodos

### send

Envía una solicitud a la canalización para procesamiento.

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**Parámetros:**
- `req` - La solicitud a enviar, debe coincidir con el tipo de entrada de la canalización

**Devuelve:**
- `Result<()>` - Ok si se puso en cola con éxito, Err si el envío falla

**Ejemplo:**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

Recibe el siguiente resultado procesado de la canalización.

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**Devuelve:**
- `Ok(Some(response))` - Un resultado procesado
- `Ok(None)` - La canalización ha terminado
- `Err(...)` - Ocurrió un error al recibir

**Ejemplo:**

```rust
loop {
    match pool.recv()? {
        Some(result) => println!("Recibido: {}", result),
        None => break,
    }
}
```

### thread_usage

Devuelve el número actual de hilos usados por la canalización.

```rust
fn thread_usage(&self) -> Result<usize>
```

**Devuelve:**
- El número total de hilos activos en todas las etapas

**Ejemplo:**

```rust
println!("Hilos activos: {}", pool.thread_usage()?);
```

### task_count

Devuelve el número de tareas pendientes para una etapa nombrada.

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**Parámetros:**
- `id` - El nombre de la etapa (según lo establecido por el atributo `#[name(...)]`)

**Devuelve:**
- El número de tareas esperando en la cola de esa etapa

**Ejemplo:**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("Profundidad de cola parser: {}", pool.task_count("parser")?);
```

## Parámetros de tipo

### Request

El tipo de entrada de la canalización. Este es el tipo aceptado por la primera etapa.

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

El tipo de salida de la canalización. Este es el tipo devuelto por la última etapa.

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## Ciclo de vida

La canalización sigue este ciclo de vida:

1. **Creada** - La macro `pipe!` devuelve un nuevo grupo
2. **Activa** - Puedes `send()` solicitudes y `recv()` resultados
3. **Drenaje** - Cuando se descarta, el grupo termina el procesamiento de tareas pendientes
4. **Terminada** - `recv()` devuelve `None` cuando el grupo se apaga

## Apagado elegante

Cuando el grupo se descarta, este:

1. Deja de aceptar nuevas solicitudes
2. Termina el procesamiento de todas las tareas encoladas
3. Apaga elegantemente todos los grupos de hilos

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // el grupo sale del alcance y se apaga elegantemente
}
```

## Monitoreo

Usa los métodos de monitoreo para rastrear la salud de la canalización:

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("stage1")]
        |req: String| -> usize { req.len() },
        #[name("stage2")]
        |req: usize| -> String { req.to_string() }
    ]?;

    // Enviar trabajo
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // Monitorear progreso
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "Hilos: {}, Stage1 pendiente: {}, Stage2 pendiente: {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
```
