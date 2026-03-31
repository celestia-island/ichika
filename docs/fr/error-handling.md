# Gestion des erreurs et réessais

Ichika fournit une gestion robuste des erreurs avec des sémantiques de réessai intégrées pour gérer les pannes transitoires.

## Propagation des erreurs

Les erreurs circulent naturellement dans le pipeline en utilisant des types `Result` :

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
            Ok(n) => format!("Résultat : {}", n),
            Err(e) => format!("Erreur : {}", e),
        }
    }
]?;
```

### Transformation de type

Lorsqu'une étape retourne un `Result`, l'étape suivante reçoit ce `Result` :

```rust
|req: String| -> anyhow::Result<usize> { ... }  // Retourne Result
|req: anyhow::Result<usize>| -> usize {         // Reçoit Result
    req.unwrap()
}
```

## Sémantiques de réessai

Ichika fournit un réessai automatique pour les opérations qui peuvent échouer de manière transitoire.

### Réessai de base

Utilisez la fonction `retry` pour réessayer une opération :

```rust
use ichika::retry;

let result = retry(|| {
    // Opération qui pourrait échouer
    Ok::<_, anyhow::Error>(42)
})?;
```

### Réessai avec politique

Contrôlez le comportement de réessai avec une `RetryPolicy` :

```rust
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

let policy = RetryPolicy {
    max_attempts: 3,
    backoff: Duration::from_millis(100),
    ..Default::default()
};

let result = retry_with(policy, || {
    // Opération avec une politique de réessai personnalisée
    Ok::<_, anyhow::Error>(42)
})?;
```

### Options RetryPolicy

```rust
pub struct RetryPolicy {
    /// Nombre maximal de tentatives de réessai
    pub max_attempts: usize,

    /// Durée de backoff initiale (backoff exponentiel appliqué)
    pub backoff: Duration,

    /// Durée de backoff maximale
    pub max_backoff: Duration,

    /// Utiliser ou non la gigue dans le calcul de backoff
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

## Utiliser le réessai dans les pipelines

### Réessai dans une étape

```rust
let pool = pipe![
    #[name("fetch")]
    |req: String| -> anyhow::Result<String> {
        // Réessayer l'opération de récupération
        retry_with(
            RetryPolicy {
                max_attempts: 3,
                backoff: Duration::from_millis(100),
                ..Default::default()
            },
            || {
                // Récupération simulée qui pourrait échouer
                if rand::random::<f32>() < 0.3 {
                    Err(anyhow::anyhow!("Erreur réseau"))
                } else {
                    Ok(format!("Récupéré : {}", req))
                }
            }
        )
    }
]?;
```

### Réessai au niveau du pipeline

Pour plus de contrôle, gérer le réessai au niveau de l'appelant :

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
                None => Err(anyhow::anyhow!("Le pipeline s'est terminé")),
            }
        }
    )
}
```

## Stratégies de récupération d'erreur

### Valeurs de secours

```rust
let pool = pipe![
    |req: String| -> anyhow::Result<i32> {
        req.parse().map_err(Into::into)
    },
    |req: anyhow::Result<i32>| -> i32 {
        req.unwrap_or(0)  // Par défaut à 0 en cas d'erreur
    }
]?;
```

### Agrégation d'erreurs

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

### Motif de disjoncteur

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

let circuit_breaker = Arc::new(AtomicBool::new(false));

let pool = pipe![
    |req: String| -> anyhow::Result<String> {
        if circuit_breaker.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("Le disjoncteur est ouvert"));
        }
        // Traiter la requête
        Ok(format!("Traité : {}", req))
    }
]?;
```

## Exemple complet

Voici un exemple complet montrant la gestion des erreurs et le réessai :

```rust
use ichika::prelude::*;
use ichika::{retry_with, RetryPolicy};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let pool = pipe![
        #[name("validate")]
        |req: String| -> anyhow::Result<i32> {
            req.parse()
                .map_err(|e| anyhow::anyhow!("Entrée invalide : {}", e))
        },
        #[name("process")]
        |req: anyhow::Result<i32>| -> anyhow::Result<i32> {
            let n = req?;
            // Simuler une panne transitoire
            if n % 3 == 0 {
                Err(anyhow::anyhow!("Erreur transitoire"))
            } else {
                Ok(n * 2)
            }
        },
        #[name("format")]
        |req: anyhow::Result<i32>| -> String {
            match req {
                Ok(n) => format!("Succès : {}", n),
                Err(e) => format!("Échec : {}", e),
            }
        }
    ]?;

    // Envoyer diverses entrées
    let inputs = vec!["10", "20", "30", "invalid", "40"];

    for input in inputs {
        pool.send(input.to_string())?;
    }

    // Collecter les résultats
    loop {
        match pool.recv()? {
            Some(result) => println!("{}", result),
            None => break,
        }
    }

    Ok(())
}
```

## Meilleures pratiques

1. **Utiliser `anyhow::Result`** pour une gestion d'erreur flexible
2. **Définir des limites de réessai appropriées** pour éviter les boucles infinies
3. **Utiliser un backoff exponentiel** pour les opérations réseau
4. **Journaliser les erreurs de manière appropriée** pour le débogage
5. **Envisager des disjoncteurs** pour les appels de services externes
6. **Rendre les erreurs informatives** - inclure le contexte sur ce qui a échoué
