# Fonctionnalités avancées

Cette section couvre les fonctionnalités et techniques avancées pour tirer le meilleur parti d'Ichika.

## Intégration asynchrone

Ichika supporte les runtimes `tokio` et `async-std`. Activez avec les fonctionnalités :

```toml
[dependencies]
ichika = { version = "0.1", features = ["tokio"] }
# ou
ichika = { version = "0.1", features = ["async-std"] }
```

### Étapes asynchrones

Mélangez les étapes synchrones et asynchrones de manière transparente :

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            req.len()  // Étape synchrone
        },
        async |req: usize| -> String {
            // Étape asynchrone - s'exécute dans le runtime tokio
            tokio::time::sleep(Duration::from_millis(100)).await;
            req.to_string()
        }
    ]?;

    Ok(())
}
```

## Créateurs de threads personnalisés

Vous pouvez personnaliser la façon dont les threads sont créés pour chaque étape :

```rust
use std::thread;

let pool = pipe![
    #[creator(|name| {
        thread::Builder::new()
            .name(name.to_string())
            .stack_size(2 * 1024 * 1024)  // Pile de 2 Mo
            .spawn(|| {
                // Logique de thread personnalisée
            })
    })]
    |req: String| -> usize {
        req.len()
    }
]?;
```

## Surveillance et observabilité

### Suivi de l'utilisation des threads

```rust
let pool = pipe![
    #[name("worker")]
    |req: String| -> usize {
        req.len()
    }
]?;

// Obtenir le nombre total de threads
let total_threads = pool.thread_usage()?;

// Obtenir les tâches en attente pour une étape nommée
let pending = pool.task_count("worker")?;

println!("Threads : {}, En attente : {}", total_threads, pending);
```

### Contrôles de santé

```rust
fn check_pool_health(pool: &impl ThreadPool) -> anyhow::Result<bool> {
    let threads = pool.thread_usage()?;
    let is_healthy = threads > 0;
    Ok(is_healthy)
}
```

## Gestion des ressources

### Arrêt gracieux

```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let running = Arc::new(AtomicBool::new(true));
let r = running.clone();

// Spawn un thread de surveillance
thread::spawn(move || {
    while r.load(Ordering::Relaxed) {
        // Surveiller la santé du pool
        thread::sleep(Duration::from_secs(1));
    }
});

// Une fois terminé, définir running à false
running.store(false, Ordering::Relaxed);
// Le pool s'arrêtera gracieusement lorsqu'il sera abandonné
```

### Considérations de mémoire

Chaque étape a une file d'attente bornée. Ajustez les tailles de file en fonction de vos contraintes de mémoire :

```rust
let pool = pipe![
    #[queue(100)]   # Petite file pour les environnements contraints en mémoire
    |req: String| -> usize {
        req.len()
    },
    #[queue(1000)]  # Plus grande file pour les étapes à haut débit
    |req: usize| -> String {
        req.to_string()
    }
]?;
```

## Modèles de pipeline

### Fan-out / Fan-in

Traiter des éléments en parallèle et collecter les résultats :

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<String> {
        req.into_iter()
            .filter(|s| !s.is_empty())
            .collect()
    },
    |req: Vec<String>| -> usize {
        req.len()
    }
]?;
```

### Traitement avec état

Utilisez `Arc<Mutex<T>>` pour des étapes avec état :

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let c = counter.clone();

let pool = pipe![
    move |req: String| -> usize {
        let mut count = c.lock().unwrap();
        *count += 1;
        println!("Éléments traités : {}", *count);
        req.len()
    }
]?;
```

### Routage conditionnel

```rust
enum Event {
    Login(String),
    Logout(String),
    Message(String, String),
}

let pool = pipe![
    |req: Event| -> String {
        match req {
            Event::Login(user) => format!("Connexion : {}", user),
            Event::Logout(user) => format!("Déconnexion : {}", user),
            Event::Message(from, msg) => format!("{} : {}", from, msg),
        }
    }
]?;
```

## Réglage des performances

### Ajustement de la taille du pool de threads

```rust
let num_cpus = num_cpus::get();

let pool = pipe![
    #[threads(num_cpus)]  // Correspond au nombre de CPU
    |req: String| -> usize {
        req.len()
    }
]?;
```

### Traitement par lots

```rust
let pool = pipe![
    |req: Vec<String>| -> Vec<usize> {
        req.par_iter()  // Utiliser rayon pour le traitement parallèle
            .map(|s| s.len())
            .collect()
    }
]?;
```

## Tester les pipelines

### Tester les étapes unitaires

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline() {
        let pool = pipe![
            |req: String| -> usize { req.len() },
            |req: usize| -> String { req.to_string() }
        ].unwrap();

        pool.send("test".to_string()).unwrap();
        let result = pool.recv().unwrap().unwrap();
        assert_eq!(result, "4");
    }
}
```

### Tests d'intégration

```rust
#[test]
fn test_error_handling() {
    let pool = pipe![
        |req: String| -> anyhow::Result<i32> {
            req.parse().map_err(Into::into)
        }
    ].unwrap();

    pool.send("invalid".to_string()).unwrap();
    // Le pipeline devrait gérer les erreurs avec grâce
}
```

## Meilleures pratiques

1. **Nommer vos étapes** pour une meilleure surveillance et débogage
2. **Utiliser des nombres de threads appropriés** - ne pas sur-souscrire le CPU
3. **Définir des tailles de file raisonnables** pour limiter l'utilisation de la mémoire
4. **Gérer les erreurs explicitement** - ne pas ignorer silencieusement les échecs
5. **Surveiller l'utilisation des ressources** en production
6. **Tester les chemins d'erreur** - pas seulement les chemins heureux
7. **Envisager la contre-pression** - que se passe-t-il quand l'aval est lent ?
8. **Utiliser async pour les étapes I/O-bound** , sync pour CPU-bound
