# Démarrage rapide

Ce guide vous aide à démarrer avec Ichika, de l'installation à votre premier pipeline.

## Installation

Ajoutez Ichika à votre `Cargo.toml` :

```toml
[dependencies]
ichika = "0.1"
```

### Fonctionnalités

Ichika supporte différents runtimes asynchrones via les fonctionnalités :

```toml
# Pour tokio (par défaut)
ichika = { version = "0.1", features = ["tokio"] }

# Pour async-std
ichika = { version = "0.1", features = ["async-std"] }

# Pour les deux runtimes
ichika = { version = "0.1", features = ["tokio", "async-std"] }
```

## Votre premier pipeline

Créons un pipeline simple qui traite des chaînes :

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // Définir un pipeline à 3 étapes
    let pool = pipe![
        // Étape 1 : Analyser la chaîne en nombre
        |req: String| -> anyhow::Result<usize> {
            req.parse::<usize>()
                .map_err(|e| anyhow::anyhow!("Échec de l'analyse : {}", e))
        },
        // Étape 2 : Doubler le nombre
        |req: anyhow::Result<usize>| -> anyhow::Result<usize> {
            req.map(|n| n * 2)
        },
        // Étape 3 : Reconvertir en chaîne
        |req: anyhow::Result<usize>| -> String {
            req.map(|n| n.to_string())
                .unwrap_or_else(|e| format!("Erreur : {}", e))
        }
    ]?;

    // Traiter des données
    pool.send("42".to_string())?;
    pool.send("100".to_string())?;
    pool.send("invalid".to_string())?;

    // Collecter les résultats
    for _ in 0..3 {
        if let Some(result) = pool.recv()? {
            println!("Résultat : {}", result);
        }
    }

    Ok(())
}
```

## Comprendre les bases

### La macro pipe!

La macro `pipe!` crée une série d'étapes de traitement. Chaque étape :

1. Reçoit l'entrée de l'étape précédente (ou de l'appel initial `send()`)
2. Traite les données dans un pool de threads
3. Passe le résultat à l'étape suivante

### Inférence de type

Ichika infère automatiquement les types qui traversent le pipeline :

```rust
let pool = pipe![
    |req: String| -> usize { req.len() },     // String -> usize
    |req: usize| -> String { req.to_string() } // usize -> String
]?;
```

### Gestion des erreurs

Chaque étape peut retourner un `Result`, et les erreurs se propagent automatiquement :

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap() // ou gérer l'erreur de manière appropriée
    }
]?;
```

## Prochaines étapes

- En savoir plus sur la [macro pipe!](./pipe-macro.md)
- Comprendre le [trait ThreadPool](./threadpool-trait.md)
- Explorer la [gestion des erreurs](./error-handling.md) en profondeur
- Voir plus [d'exemples](./examples.md)
