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
        if pool.recv()?.is_some() {
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
    std::thread::sleep(Duration::from_millis(400));

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
    std::thread::sleep(Duration::from_millis(400));

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

#[test]
fn test_task_count_is_per_stage() -> Result<()> {
    // task_count(stage) must report the backlog of the *named* stage, not a global total.
    // A slow single-worker stage lets a backlog accumulate so we can observe a non-zero,
    // stage-scoped count, while an unknown stage reports 0.
    let pool = pipe![
        slow_stage: (max_threads: 1) |req: String| -> String {
            std::thread::sleep(Duration::from_millis(100));
            Ok(req)
        },
    ]?;

    // Allow the daemon to spin up.
    std::thread::sleep(Duration::from_millis(200));

    // Queue several requests; with max_threads:1 they back up in this stage.
    for i in 0..5 {
        pool.send(format!("req-{}", i))?;
    }
    // Give the daemon a loop tick to observe the backlog.
    std::thread::sleep(Duration::from_millis(150));

    let known = pool.task_count("slow_stage")?;
    assert!(
        known > 0,
        "expected pending tasks for 'slow_stage', got {}",
        known
    );

    // An unknown stage must report 0, proving the count is stage-scoped.
    let unknown = pool.task_count("does-not-exist")?;
    assert_eq!(unknown, 0);

    // Drain.
    std::thread::sleep(Duration::from_millis(900));
    while pool.recv()?.is_some() {}

    Ok(())
}

#[test]
fn test_leading_paren_with_comma_is_global() -> Result<()> {
    // Regression guard for the disambiguation between a *global* constraint
    // `(max_threads: N),` and a per-step constraint on the first closure
    // `(max_threads: N) |req| ...`. A leading paren followed by a comma must be
    // parsed as GLOBAL. With global max_threads:1 the whole pool is hard-capped
    // at a single live worker at any instant, so thread_usage can never exceed 1
    // no matter how large the backlog grows. A fast first stage feeding a slow
    // second stage forces a backlog that an unconstrained pool would scale up
    // for — so this invariant only holds when the cap is applied globally.
    let pool = pipe![
        (max_threads: 1),
        |req: String| -> String { Ok(req) },
        |req: String| -> usize {
            std::thread::sleep(Duration::from_millis(40));
            Ok(req.len())
        }
    ]?;

    std::thread::sleep(Duration::from_millis(200));

    for i in 0..16 {
        pool.send(format!("req-{}", i))?;
    }
    std::thread::sleep(Duration::from_millis(350));

    let usage = pool.thread_usage()?;
    assert!(
        usage <= 1,
        "global max_threads:1 must cap total live workers, got {}",
        usage
    );

    std::thread::sleep(Duration::from_millis(1500));
    while pool.recv()?.is_some() {}

    Ok(())
}

#[test]
fn test_first_stage_constraint_is_not_global() -> Result<()> {
    // Regression for the actual bug: a per-step constraint on the FIRST closure
    // — `(max_threads: 1) |req| ...` with no comma — must bind to that stage
    // only, NOT leak into a global cap. Here stage 1 is capped at 1 worker while
    // stage 2 is unconstrained. We feed a fast stage 1 that floods a slow stage
    // 2; if the cap were (incorrectly) global the pool would be hard-limited to
    // one live worker total. On a multi-core host the unconstrained stage 2 must
    // be free to scale beyond 1, which is impossible under a global cap.
    let pool = pipe![
        (max_threads: 1) |req: String| -> String { Ok(req) },
        |req: String| -> usize {
            std::thread::sleep(Duration::from_millis(40));
            Ok(req.len())
        }
    ]?;

    std::thread::sleep(Duration::from_millis(200));

    for i in 0..16 {
        pool.send(format!("req-{}", i))?;
    }
    std::thread::sleep(Duration::from_millis(350));

    let usage = pool.thread_usage()?;
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    if cpus > 1 {
        // Under the bug this is hard-capped at 1; under the fix stage 2 scales up.
        assert!(
            usage >= 2,
            "per-step cap on stage 1 leaked into a global cap (usage={}, cpus={})",
            usage,
            cpus
        );
    }

    std::thread::sleep(Duration::from_millis(1500));
    while pool.recv()?.is_some() {}

    Ok(())
}
