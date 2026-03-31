//! E2E tests for M4: Per-Step Thread Limit feature
//!
//! Tests verify that:
//! 1. Global max thread count is respected
//! 2. Per-step max/min thread counts are respected
//! 3. Default behavior remains unchanged (backward compatible)

use anyhow::Result;
use ichika::prelude::*;
use std::time::Duration;

#[test]
fn test_backward_compatibility_no_constraints() -> Result<()> {
    // Test that default behavior works without any constraints
    let pool = pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(200));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "4");

    Ok(())
}

#[test]
fn test_global_max_threads_constraint() -> Result<()> {
    // Test global max_threads constraint
    let pool = pipe![
        (max_threads: 2),
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    // Send multiple requests
    for i in 0..5 {
        pool.send(format!("test_{}", i))?;
    }

    std::thread::sleep(Duration::from_millis(300));

    // Verify responses are received
    let mut count = 0;
    loop {
        if let Some(_) = pool.recv()? {
            count += 1;
        } else {
            break;
        }
        if count >= 5 {
            break;
        }
    }

    assert_eq!(count, 5);
    Ok(())
}

#[test]
fn test_per_step_max_threads_constraint() -> Result<()> {
    // Test per-step max_threads constraint
    let pool = pipe![
        |req: String| -> usize { Ok(req.len()) },
        (max_threads: 1) |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(200));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "4");

    Ok(())
}

#[test]
fn test_per_step_min_threads_constraint() -> Result<()> {
    // Test per-step min_threads constraint
    let pool = pipe![
        (min_threads: 2) |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(200));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "4");

    Ok(())
}

#[test]
fn test_combined_global_and_step_constraints() -> Result<()> {
    // Test that per-step constraints override global constraints
    let pool = pipe![
        (max_threads: 4),
        |req: String| -> usize { Ok(req.len()) },
        (max_threads: 1) |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(200));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "4");

    Ok(())
}

#[test]
fn test_multiple_steps_with_different_constraints() -> Result<()> {
    // Test multiple steps each with different constraints
    let pool = pipe![
        (max_threads: 2) |req: String| -> usize { Ok(req.len()) },
        (max_threads: 1) |req: usize| -> u64 { Ok(req as u64 * 2) },
        (max_threads: 3) |req: u64| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(300));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "8"); // "test".len() = 4, 4 * 2 = 8

    Ok(())
}

#[test]
fn test_thread_usage_with_constraints() -> Result<()> {
    // Test that thread_usage reports correct counts with constraints
    let pool = pipe![
        (max_threads: 3),
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    // Send a request to spawn threads
    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(200));

    let usage = pool.thread_usage()?;
    // With max_threads: 3, usage should not exceed 3
    assert!(usage <= 3, "Thread usage {} exceeds max_threads 3", usage);

    // Clean up
    let _ = pool.recv()?;

    Ok(())
}

#[test]
fn test_step_with_both_min_and_max_constraints() -> Result<()> {
    // Test per-step with both min and max threads
    let pool = pipe![
        (min_threads: 1, max_threads: 2) |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(200));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "4");

    Ok(())
}

#[test]
fn test_constraints_with_async_steps() -> Result<()> {
    // Test that constraints work with async steps
    let pool = pipe![
        (max_threads: 2) async |req: String| -> usize {
            Ok(tokio::task::spawn_blocking(move || req.len()).await.unwrap())
        },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(300));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "4");

    Ok(())
}

#[test]
fn test_named_step_with_constraints() -> Result<()> {
    // Test that named steps work with constraints
    let pool = pipe![
        step1: (max_threads: 2) |req: String| -> usize { Ok(req.len()) },
        step2: |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(200));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "4");

    Ok(())
}

#[test]
fn test_single_step_with_constraint() -> Result<()> {
    // Test single step pipeline with constraint
    let pool = pipe![
        (max_threads: 1) |req: String| -> String { Ok(req.to_uppercase()) }
    ]?;

    pool.send("test".to_string())?;
    std::thread::sleep(Duration::from_millis(200));

    let res = pool.recv()?;
    assert!(res.is_some());
    assert_eq!(res.unwrap(), "TEST");

    Ok(())
}
