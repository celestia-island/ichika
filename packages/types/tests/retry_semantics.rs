use anyhow::Result;
use ichika::pipe;
use ichika::pool::ThreadPool;
use ichika::RetryPolicy;

#[test]
fn test_basic_retry_discards_after_max_attempts() -> Result<()> {
    // Status::Retry retries with the same input using the default policy.
    // Once max attempts are exceeded, the request is silently discarded (no output).
    let pool = pipe![
        retry_step: |_req: String| -> String {
            ichika::retry::<String, anyhow::Error>()
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("test".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(500));

    // After max attempts are exhausted, Retry discards the item – nothing arrives.
    let mut received = 0;
    loop {
        if let Some(_) = pool.recv()? {
            received += 1;
        } else {
            break;
        }
    }
    assert_eq!(received, 0);

    Ok(())
}

#[test]
fn test_retry_with_fallback_value() -> Result<()> {
    // RetryWith retries the step with the same original request.
    // After max_attempts retries, the provided fallback (Output-typed) value is sent.
    let pool = pipe![
        custom_retry: |_req: String| -> String {
            ichika::Status::RetryWith(
                RetryPolicy { max_attempts: 2, delay_ms: 50 },
                0,
                "done".to_string(),
            )
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("test".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(400));

    let mut received = 0;
    loop {
        if let Some(res) = pool.recv()? {
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
fn test_retry_always_discards_on_max_exceeded() -> Result<()> {
    // When a step always returns Retry, max attempts are eventually exhausted
    // and the item is silently dropped (received == 0).
    let pool = pipe![
        always_fail: |_req: String| -> String {
            ichika::retry::<String, anyhow::Error>()
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("original".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut received = 0;
    loop {
        if let Some(_) = pool.recv()? {
            received += 1;
        } else {
            break;
        }
    }
    assert_eq!(received, 0);

    Ok(())
}

#[test]
fn test_retry_with_delay_respected() -> Result<()> {
    // Verify that the delay_ms in RetryWith is actually observed.
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
    std::thread::sleep(std::time::Duration::from_millis(600));

    let mut received = 0;
    loop {
        if let Some(res) = pool.recv()? {
            assert_eq!(res, "delayed");
            received += 1;
        } else {
            break;
        }
    }

    let elapsed = start.elapsed();
    // Two retries x 150 ms == 300 ms minimum before fallback is sent.
    assert!(elapsed >= std::time::Duration::from_millis(250));
    assert_eq!(received, 1);

    Ok(())
}

#[test]
fn test_retry_with_sends_fallback_when_condition_never_met() -> Result<()> {
    // A step that always returns RetryWith when the input is empty
    // eventually sends its fallback output once max retries are exhausted.
    let pool = pipe![
        step: |req: String| -> String {
            if req.is_empty() {
                ichika::Status::RetryWith(
                    RetryPolicy { max_attempts: 3, delay_ms: 20 },
                    0,
                    "fallback".to_string(),
                )
            } else {
                ichika::Status::Next(format!("ok:{}", req.len()))
            }
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    // Empty string retries and eventually emits the fallback.
    pool.send("".to_string())?;
    // Non-empty string succeeds immediately.
    pool.send("hello".to_string())?;

    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut results = Vec::new();
    loop {
        if let Some(res) = pool.recv()? {
            results.push(res);
        } else {
            break;
        }
    }

    assert_eq!(results.len(), 2);
    assert!(results.contains(&"fallback".to_string()));
    assert!(results.contains(&"ok:5".to_string()));

    Ok(())
}

#[test]
fn test_retry_immediate_success() -> Result<()> {
    // When no retry is needed, the request is processed on the first try.
    let pool = pipe![
        no_retry_needed: |req: String| -> String {
            Ok(format!("processed:{}", req))
        },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    pool.send("valid".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut received = 0;
    loop {
        if let Some(res) = pool.recv()? {
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
fn test_retry_with_fallback_in_multi_step_pipeline() -> Result<()> {
    // RetryWith produces its fallback value which then flows into the next step.
    let pool = pipe![
        first: |req: String| -> String {
            if req.is_empty() {
                ichika::Status::RetryWith(
                    RetryPolicy { max_attempts: 1, delay_ms: 20 },
                    0,
                    "nonempty".to_string(),
                )
            } else {
                ichika::Status::Next(req)
            }
        },
        second: |req: String| -> usize { Ok(req.len()) },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    pool.send("".to_string())?;     // triggers RetryWith fallback "nonempty"
    pool.send("hi".to_string())?;  // processed directly

    std::thread::sleep(std::time::Duration::from_millis(300));

    let mut results: Vec<usize> = Vec::new();
    loop {
        if let Some(res) = pool.recv()? {
            results.push(res);
        } else {
            break;
        }
    }

    // "nonempty" -> len = 8, "hi" -> len = 2
    assert_eq!(results.len(), 2);
    assert!(results.contains(&8));
    assert!(results.contains(&2));

    Ok(())
}
