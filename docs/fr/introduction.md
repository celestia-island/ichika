# Introduction

**Ichika** est une bibliothèque de macros procédurales Rust pour construire des pipelines basés sur des pools de threads avec gestion automatique des erreurs, sémantique de réessai et support d'arrêt gracieux.

## Vue d'ensemble

Ichika fournit une macro `pipe!` puissante qui vous permet de définir des pipelines de traitement multi-étapes complexes où chaque étape s'exécute dans son propre pool de threads. La macro gère tout le code boilerplate pour créer des pools de threads, configurer des canaux de communication et coordonner les étapes.

## Fonctionnalités clés

- **Syntaxe de pipeline déclarative** : Définissez des pipelines de traitement complexes avec une syntaxe de macro propre et expressive
- **Gestion automatique des pools de threads** : Chaque étape obtient son propre pool de threads dédié
- **Propagation des erreurs** : Gestion des erreurs intégrée avec des types `Result` dans tout le pipeline
- **Sémantique de réessai** : Politiques de réessai configurables pour gérer les échecs transitoires
- **Indépendance du runtime asynchrone** : Fonctionne avec `tokio` et `async-std`
- **Arrêt gracieux** : Nettoyage approprié lorsque le pipeline est abandonné
- **Surveillance** : Statistiques d'utilisation des threads et comptage des tâches intégrés

## Exemple simple

```rust
use ichika::prelude::*;

fn main() -> anyhow::Result<()> {
    // Créer un pipeline simple à 2 étapes
    let pool = pipe![
        |req: String| -> usize {
            Ok(req.len())
        },
        |req: usize| -> String {
            Ok(req.to_string())
        }
    ]?;

    // Envoyer des requêtes
    pool.send("hello".to_string())?;
    pool.send("world".to_string())?;

    // Collecter les résultats
    while let Some(result) = pool.recv()? {
        println!("Got: {}", result);
    }

    Ok(())
}
```

## Cas d'utilisation

Ichika est particulièrement utile pour :

- **Pipelines de traitement de données** : Workflows de transformation de données multi-étapes
- **Traitement des requêtes API** : Traitement des requêtes via plusieurs étapes de validation/transformation
- **Traitement d'événements** : Systèmes pilotés par événements avec traitement par étapes
- **Traitements par lots** : Traitement parallèle avec concurrence configurable par étape
- **Microservices** : Communication de service interne avec files d'attente bornées

## Philosophie de conception

Ichika suit ces principes :

1. **Sécurité d'abord** : Exploite le système de types de Rust pour des garanties au moment de la compilation
2. **API ergonomique** : Minimise le code boilerplate tout en maintenant la flexibilité
3. **Abstractions sans coût** : Aucune surcharge d'exécution au-delà de ce qui est nécessaire
4. **Contrôle explicite** : Donne aux utilisateurs un contrôle précis sur les pools de threads et les files d'attente

## État du projet

Ichika est actuellement en développement actif. L'API peut changer entre les versions, mais nous nous efforçons de maintenir la compatibilité descendante dans la mesure du possible.

## Licence

Ichika est sous licence MIT. Voir [LICENSE](https://github.com/celestia-island/ichika/blob/master/LICENSE) pour plus de détails.
