# Trait ThreadPool

Le trait `ThreadPool` définit l'interface pour tous les pools de pipelines créés par la macro `pipe!`.

## Définition du trait

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

## Méthodes

### send

Envoie une requête au pipeline pour traitement.

```rust
fn send(&self, req: Self::Request) -> Result<()>
```

**Paramètres :**
- `req` - La requête à envoyer, doit correspondre au type d'entrée du pipeline

**Retourne :**
- `Result<()>` - Ok si mis en file avec succès, Err si l'envoi échoue

**Exemple :**

```rust
let pool = pipe![
    |req: String| -> usize { req.len() }
]?;

pool.send("hello".to_string())?;
```

### recv

Reçoit le prochain résultat traité du pipeline.

```rust
fn recv(&self) -> Result<Option<Self::Response>>
```

**Retourne :**
- `Ok(Some(response))` - Un résultat traité
- `Ok(None)` - Le pipeline s'est terminé
- `Err(...)` - Une erreur s'est produite lors de la réception

**Exemple :**

```rust
loop {
    match pool.recv()? {
        Some(result) => println!("Reçu : {}", result),
        None => break,
    }
}
```

### thread_usage

Retourne le nombre actuel de threads utilisés par le pipeline.

```rust
fn thread_usage(&self) -> Result<usize>
```

**Retourne :**
- Le nombre total de threads actifs dans toutes les étapes

**Exemple :**

```rust
println!("Threads actifs : {}", pool.thread_usage()?);
```

### task_count

Retourne le nombre de tâches en attente pour une étape nommée.

```rust
fn task_count(&self, id: impl ToString) -> Result<usize>
```

**Paramètres :**
- `id` - Le nom de l'étape (tel que défini par l'attribut `#[name(...)]`)

**Retourne :**
- Le nombre de tâches en attente dans la file de cette étape

**Exemple :**

```rust
let pool = pipe![
    #[name("parser")]
    |req: String| -> usize { req.len() }
]?;

pool.send("test".to_string())?;
println!("Profondeur de file parser : {}", pool.task_count("parser")?);
```

## Paramètres de type

### Request

Le type d'entrée du pipeline. C'est le type accepté par la première étape.

```rust
let pool: impl ThreadPool<Request = String, Response = usize> = pipe![
    |req: String| -> usize { req.len() }
]?;
```

### Response

Le type de sortie du pipeline. C'est le type retourné par la dernière étape.

```rust
let pool: impl ThreadPool<Request = String, Response = String> = pipe![
    |req: String| -> usize { req.len() },
    |req: usize| -> String { req.to_string() }
]?;
```

## Cycle de vie

Le pipeline suit ce cycle de vie :

1. **Créé** - La macro `pipe!` retourne un nouveau pool
2. **Actif** - Vous pouvez `send()` des requêtes et `recv()` des résultats
3. **Vidage** - Lorsqu'il est abandonné, le pool termine le traitement des tâches en attente
4. **Terminé** - `recv()` retourne `None` lorsque le pool est arrêté

## Arrêt gracieux

Lorsque le pool est abandonné, il :

1. Arrête d'accepter de nouvelles requêtes
2. Termine le traitement de toutes les tâches en file
3. Arrête gracieusement tous les pools de threads

```rust
{
    let pool = pipe![
        |req: String| -> usize { req.len() }
    ]?;

    pool.send("hello".to_string())?;
    // le pool sort de la portée et s'arrête gracieusement
}
```

## Surveillance

Utilisez les méthodes de surveillance pour suivre la santé du pipeline :

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

    // Envoyer du travail
    for i in 0..100 {
        pool.send(format!("request-{}", i))?;
    }

    // Surveiller les progrès
    loop {
        let threads = pool.thread_usage()?;
        let stage1_pending = pool.task_count("stage1")?;
        let stage2_pending = pool.task_count("stage2")?;

        println!(
            "Threads : {}, Stage1 en attente : {}, Stage2 en attente : {}",
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
