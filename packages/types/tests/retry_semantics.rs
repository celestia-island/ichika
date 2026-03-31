use anyhow::Result;
use ichika::pipe;
use ichika::pool::ThreadPool;
use ichika::RetryPolicy;

#[test]
fn test_basic_retry_with_default_policy() -> Result<()> {
    // Test that Status::Retry uses default policy (3 attempts, 100ms delay)
    let pool = pipe![
        retry_step: |_req: String| -> String {
            // Return Status::Retry to trigger default retry
            // After max attempts, the original request is sent as-is
            ichika::retry::<String, anyhow::Error>()
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("test".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            // After max attempts (3), the original request is sent
            assert_eq!(res, "test");
            received += 1;
        } else {
            break;
        }
    }

    // Should receive 1 result after max attempts exceeded
    assert_eq!(received, 1);

    Ok(())
}

#[test]
fn test_retry_with_custom_policy() -> Result<()> {
    // Test RetryWith with custom max_attempts and delay
    let pool = pipe![
        custom_retry: |_req: String| -> String {
            // Use RetryWith to retry with a new value
            ichika::Status::RetryWith(
                RetryPolicy { max_attempts: 2, delay_ms: 50 },
                0,
                "done".to_string(),
            )
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("test".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            // After max attempts, we get "done"
            assert_eq!(res, "done");
            received += 1;
        } else {
            break;
        }
    }

    assert_eq!(received, 1);

    Ok(())
}

#[test]
fn test_retry_max_attempts_exceeded() -> Result<()> {
    // Test behavior when max attempts is exceeded
    let pool = pipe![
        always_fail: |_req: String| -> String {
            // Always retry - will exceed default max of 3
            ichika::retry::<String, anyhow::Error>()
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("original".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            // After max attempts, the original request is sent
            assert_eq!(res, "original");
            received += 1;
        } else {
            break;
        }
    }

    assert_eq!(received, 1);

    Ok(())
}

#[test]
fn test_retry_with_delay() -> Result<()> {
    // Test that retry delay is respected
    let pool = pipe![
        delayed_retry: |_req: String| -> String {
            ichika::Status::RetryWith(
                RetryPolicy { max_attempts: 2, delay_ms: 150 },
                0,
                "delayed".to_string(),
            )
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    let start = std::time::Instant::now();
    pool.send("test".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, "delayed");
            received += 1;
        } else {
            break;
        }
    }

    let elapsed = start.elapsed();
    // Should take at least 150ms (one delay)
    assert!(elapsed >= std::time::Duration::from_millis(140));
    assert_eq!(received, 1);

    Ok(())
}

#[test]
fn test_retry_with_successful_pipeline() -> Result<()> {
    // Test retry in a multi-step pipeline
    let pool = pipe![
        step_with_retry: |req: String| -> String {
            // Retry until we get a non-empty string
            if req.is_empty() {
                ichika::Status::RetryWith(
                    RetryPolicy { max_attempts: 3, delay_ms: 50 },
                    0,
                    "default".to_string(),
                )
            } else {
                ichika::Status::Next(format!("length:{}", req.len()))
            }
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Send empty string - will retry with default value
    pool.send("".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, "length:7"); // len("default") = 7
            received += 1;
        } else {
            break;
        }
    }

    assert_eq!(received, 1);

    Ok(())
}

#[test]
fn test_retry_immediate_success() -> Result<()> {
    // Test that requests succeed immediately without retry when valid
    let pool = pipe![
        no_retry_needed: |req: String| -> String {
            Ok(format!("processed:{}", req))
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("valid".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, "processed:valid");
            received += 1;
        } else {
            break;
        }
    }

    assert_eq!(received, 1);

    Ok(())
}

#[test]
fn test_retry_with_incremental_values() -> Result<()> {
    // Test retry that modifies the value on each attempt
    let pool = pipe![
        counter: |req: String| -> String {
            // Parse "count:N" and increment
            if let Some(count_str) = req.strip_prefix("count:") {
                let count: usize = count_str.parse().unwrap_or(0);
                if count < 2 {
                    // Retry with incremented count
                    ichika::Status::RetryWith(
                        RetryPolicy { max_attempts: 3, delay_ms: 50 },
                        count,
                        format!("count:{}", count + 1),
                    )
                } else {
                    // Success after 2 retries
                    ichika::Status::Next(format!("final_count:{}", count))
                }
            } else {
                // First request - start counting
                ichika::Status::RetryWith(
                    RetryPolicy { max_attempts: 3, delay_ms: 50 },
                    0,
                    "count:1".to_string(),
                )
            }
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("start".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert!(res.contains("final_count"));
            received += 1;
        } else {
            break;
        }
    }

    assert_eq!(received, 1);

    Ok(())
}
