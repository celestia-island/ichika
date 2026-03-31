# Introducción

**Ichika** es una biblioteca de macros procedurales de Rust para construir canalizaciones basadas en grupos de hilos con manejo automático de errores, semántica de reintento y soporte de apagado elegante.

## Resumen

Ichika proporciona una poderosa macro `pipe!` que le permite definir canalizaciones de procesamiento complejas de múltiples etapas donde cada etapa se ejecuta en su propio grupo de hilos. La macro maneja todo el código repetitivo de crear grupos de hilos, configurar canales de comunicación y coordinar entre etapas.

## Características clave

- **Sintaxis de canalización declarativa**: Defina canalizaciones de procesamiento complejas usando una sintaxis de macro limpia y expresiva
- **Gestión automática de grupos de hilos**: Cada etapa obtiene su propio grupo de hilos dedicado
- **Propagación de errores**: Manejo de errores integrado con tipos `Result` en toda la canalización
- **Semántica de reintento**: Políticas de reintento configurables para manejar fallas transitorias
- **Agóstico de tiempo de ejecución asíncrono**: Funciona tanto con `tokio` como con `async-std`
- **Apagado elegante**: Limpieza adecuada cuando se descarta la canalización
- **Monitoreo**: Estadísticas de uso de hilos y conteo de tareas integrados

## Ejemplo simple

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // Crear una canalización simple de 2 etapas
    let pool = pipe![
        |req: String| -> usize {
            Ok(req.len())
        },
        |req: usize| -> String {
            Ok(req.to_string())
        }
    ]?;

    // Enviar solicitudes
    pool.send("hello".to_string())?;
    pool.send("world".to_string())?;

    // Recopilar resultados
    while let Some(result) = pool.recv()? {
        println!("Got: {}", result);
    }

    Ok(())
}
```

## Casos de uso

Ichika es particularmente útil para:

- **Canalizaciones de procesamiento de datos**: Flujos de trabajo de transformación de datos de múltiples etapas
- **Manejo de solicitudes API**: Procesamiento de solicitudes a través de múltiples etapas de validación/transformación
- **Procesamiento de eventos**: Construcción de sistemas controlados por eventos con procesamiento por etapas
- **Trabajos por lotes**: Procesamiento paralelo con concurrencia configurable por etapa
- **Microservicios**: Comunicación de servicio interno con colas limitadas

## Filosofía de diseño

Ichika sigue estos principios:

1. **Seguridad primero**: Aprovecha el sistema de tipos de Rust para garantías en tiempo de compilación
2. **API ergonómica**: Minimiza el código repetitivo mientras mantiene la flexibilidad
3. **Abstracciones de costo cero**: No hay sobrecarga de tiempo de ejecución más allá de lo necesario
4. **Control explícito**: Da a los usuarios un control de grano fino sobre los grupos de hilos y las colas

## Estado del proyecto

Ichika está actualmente en desarrollo activo. La API puede cambiar entre versiones, pero nos esforzamos por mantener la compatibilidad con versiones anteriores siempre que sea posible.

## Licencia

Ichika está licenciado bajo la Licencia MIT. Vea [LICENSE](https://github.com/celestia-island/ichika/blob/master/LICENSE) para más detalles.
