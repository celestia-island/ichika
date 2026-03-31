# Exemples

Cette page contient des exemples pratiques démontrant diverses fonctionnalités d'Ichika.

## Table des matières

- [Pipeline synchrone de base](#pipeline-synchrone-de-base)
- [Pipeline asynchrone de base](#pipeline-asynchrone-de-base)
- [Gestion des erreurs](#gestion-des-erreurs)
- [Arrêt gracieux](#arrêt-gracieux)
- [Surveillance de l'utilisation des threads](#surveillance-de-lutilisation-des-threads)
- [Pipeline avec payload tuple](#pipeline-avec-payload-tuple)

## Pipeline synchrone de base

Un exemple minimal montrant un pipeline synchrone simple à 2 étapes :

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("Conversion de '{}' en longueur", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("Conversion de la longueur {} en chaîne", req);
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
            Some(output) => log::info!("Reçu : {}", output),
            None => break,
        }
    }

    Ok(())
}
```

## Pipeline asynchrone de base

Exemple utilisant des étapes asynchrones avec tokio :

```rust
use ichika::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        |req: String| -> usize {
            log::info!("Étape 1 : {}", req);
            req.len()
        },
        async |req: usize| -> String {
            log::info!("Étape 2 : traitement {}", req);
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    pool.send("async".to_string())?;
    pool.send("pipeline".to_string())?;

    tokio::time::sleep(Duration::from_secs(1)).await;

    loop {
        match pool.recv()? {
            Some(result) => println!("Résultat : {}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Gestion des erreurs

Démontrant la propagation des erreurs à travers le pipeline :

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("parse")]
        |req: String| -> anyhow::Result<i32> {
            log::info!("Analyse : {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("Traitement : {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("Résultat : {}", n),
                Err(e) => format!("Erreur : {}", e),
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

## Arrêt gracieux

Démontrant le nettoyage approprié lorsque le pipeline est abandonné :

```rust
use ichika::prelude::*;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    {
        let pool = pipe![
            |req: String| -> usize {
                log::info!("Traitement : {}", req);
                std::thread::sleep(Duration::from_millis(50));
                req.len()
            }
        ]?;

        // Envoyer du travail
        for i in 0..10 {
            pool.send(format!("request-{}", i))?;
        }

        // Donner du temps pour le traitement
        std::thread::sleep(Duration::from_millis(200));

        // Le pool sera abandonné et s'arrêtera gracieusement
        log::info!("Le pool sort de la portée...");
    }

    log::info!("Le pool s'est arrêté gracieusement");

    Ok(())
}
```

## Surveillance de l'utilisation des threads

Suivi de l'utilisation des threads et des comptes de tâches :

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

    // Envoyer du travail
    for i in 0..50 {
        pool.send(format!("request-{}", i))?;
    }

    // Surveiller les progrès
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "Threads : {}, Stage1 : {}, Stage2 : {}",
            threads, stage1_pending, stage2_pending
        );

        if stage1_pending == 0 && stage2_pending == 0 {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));
    }

    println!("Toutes les tâches terminées");

    Ok(())
}
```

## Pipeline avec payload tuple

Travailler avec des payloads tuple :

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> (String, usize) {
            let len = req.len();
            (req, len)
        },
        |req: (String, usize)| -> String {
            format!("'{}' a une longueur de {}", req.0, req.1)
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

## Exécuter les exemples

Tous les exemples sont disponibles dans le référentiel :

```bash
# Exécuter un exemple spécifique
cargo run --example basic_sync_chain

# Exécuter avec la journalisation
RUST_LOG=info cargo run --example basic_sync_chain

# Exécuter l'exemple async
cargo run --example basic_async_chain --features tokio
```

## Plus d'exemples

Consultez le répertoire `examples/` dans le référentiel pour plus d'exemples complets :

- `basic_sync_chain.rs` - Pipeline synchrone
- `basic_async_chain.rs` - Pipeline asynchrone
- `error_handling.rs` - Propagation des erreurs
- `graceful_shutdown_drop.rs` - Nettoyage lors de l'abandon
- `monitoring_thread_usage.rs` - API de surveillance
- `tuple_payload_pipeline.rs` - Types de payload complexes
- `status_exit_demo.rs` - Gestion de l'état et de la sortie
