# La macro pipe!

La macro `pipe!` est au cœur d'Ichika. Elle transforme une séquence de fermetures en un pipeline de traitement multi-étapes entièrement fonctionnel.

## Syntaxe de base

```rust
let pool = pipe![
    closure1,
    closure2,
    closure3,
    // ... plus de fermetures
]?;
```

Chaque fermeture représente une étape de traitement dans votre pipeline.

## Signatures des fermetures

Chaque fermeture doit suivre ces règles :

1. **Accepter exactement un paramètre** - l'entrée de l'étape précédente
2. **Retourner un type** - qui devient l'entrée de l'étape suivante
3. Être `Clone + Send + 'static` - requis pour l'exécution du pool de threads

### Exemples de signatures

```rust
|req: String| -> usize {
    req.len()
}

|req: usize| -> anyhow::Result<String> {
    Ok(req.to_string())
}

|req: anyhow::Result<MyData>| -> MyOutput {
    // Gérer le Result
}
```

## Inférence de type

Ichika connecte automatiquement le type de sortie d'une étape au type d'entrée de la suivante :

```rust
let pool = pipe![
    |req: String| -> usize {        // Étape 1 : String -> usize
        req.len()
    },
    |req: usize| -> String {         // Étape 2 : usize -> String
        req.to_string()
    },
    |req: String| -> bool {          // Étape 3 : String -> bool
        !req.is_empty()
    }
]?;
```

## Attributs d'étape

Vous pouvez configurer des étapes individuelles en utilisant des attributs :

### Configuration du pool de threads

```rust
let pool = pipe![
    #[threads(4)]                    // Utiliser 4 threads pour cette étape
    |req: String| -> usize {
        req.len()
    },
    #[threads(2)]                    // Utiliser 2 threads pour cette étape
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

### Configuration de la file d'attente

```rust
let pool = pipe![
    #[queue(100)]                    // Capacité de file de 100
    |req: String| -> usize {
        req.len()
    }
]?;
```

### Étapes nommées

```rust
let pool = pipe![
    #[name("parser")]                // Nommer l'étape pour la surveillance
    |req: String| -> usize {
        req.len()
    },
    #[name("formatter")]
    |req: usize| -> String {
        req.to_string()
    }
]?;

// Interroger le nombre de tâches pour une étape nommée
let count = pool.task_count("parser")?;
```

## Pipelines avec branchement

Vous pouvez créer un branchement conditionnel dans votre pipeline :

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<Either<usize, String>> {
        if req.parse::<usize>().is_ok() {
            Ok(Either::Left(req.parse::<usize>()?))
        } else {
            Ok(Either::Right(req))
        }
    },
    // Gérer chaque branche
    |req: Either<usize, String>| -> String {
        match req {
            Either::Left(n) => format!("Nombre : {}", n),
            Either::Right(s) => format!("Chaîne : {}", s),
        }
    }
]?;
```

## Étapes asynchrones

Avec les fonctionnalités appropriées, vous pouvez utiliser des étapes asynchrones :

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()
        },
        async |req: usize| -> String {
            // S'exécute dans le runtime async
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## Contraintes globales

Vous pouvez définir des contraintes globales pour l'ensemble du pipeline :

```rust
let pool = pipe![
    #[global_threads(8)]             // Nombre de threads par défaut pour toutes les étapes
    #[global_queue(1000)]            // Capacité de file par défaut
    |req: String| -> usize {
        req.len()
    },
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## Exemple complet

Voici un exemple plus réaliste montrant plusieurs fonctionnalités :

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let pool = pipe![
        #[name("parse")]
        #[threads(2)]
        |req: String| -> anyhow::Result<i32> {
            log::info!("Analyse : {}", req);
            req.parse().map_err(Into::into)
        },
        #[name("process")]
        #[threads(4)]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            log::info!("Traitement : {}", n);
            Ok(n * 2)
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => {
                    log::info!("Formatage : {}", n);
                    format!("Résultat : {}", n)
                }
                Err(e) => {
                    log::error!("Erreur : {}", e);
                    format!("Erreur : {}", e)
                }
            }
        }
    ]?;

    // Surveiller l'utilisation des threads
    println!("Utilisation des threads : {}", pool.thread_usage()?);

    Ok(())
}
```
