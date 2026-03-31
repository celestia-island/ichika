use anyhow::Result;
use ichika::pipe;
use ichika::pool::ThreadPool;

#[test]
fn test_basic_pipeline_with_result_wrapping() -> Result<()> {
    // Test that Result wrapping works via IntoStatus
    let pool = pipe![
        step1: |req: String| -> usize {
            // Using ? operator requires returning Result
            // But the closure returns T, which becomes Status<T, ()>
            Ok(req.len())
        },
        step2: |req: usize| -> String { Ok(format!("length: {}", req)) },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Send valid inputs
    pool.send("hello".to_string())?;
    pool.send("world".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(_) = res {
            received += 1;
        } else {
            break;
        }
    }

    assert_eq!(received, 2);

    Ok(())
}

#[test]
fn test_panic_handling() -> Result<()> {
    // Test that panics are logged and don't crash the pool
    let pool = pipe![
        risky_step: |req: String| -> usize {
            // This will panic for empty strings
            if req.is_empty() {
                panic!("Empty string not allowed")
            }
            Ok(req.len())
        },
        formatter: |req: usize| -> String { Ok(format!("result: {}", req)) },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Valid input - should succeed
    pool.send("test".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    // This will panic in risky_step - thread will log and exit
    pool.send("".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Another valid input - pool should still work
    pool.send("another".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(_) = res {
            received += 1;
        } else {
            break;
        }
    }

    // Should receive 2 results (test and another)
    assert_eq!(received, 2);

    Ok(())
}

#[test]
fn test_error_handling_with_panic() -> Result<()> {
    // Test different error scenarios
    let pool = pipe![
        validator: |req: String| -> String {
            // Validate and transform
            if req.starts_with("error:") {
                panic!("Error path: {}", req);
            }
            Ok(req.to_uppercase())
        },
        processor: |req: String| -> String { Ok(format!("processed: {}", req)) },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Normal case
    pool.send("normal".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Error case - will panic
    pool.send("error:invalid".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200)); // Allow thread recovery time

    // Another normal case
    pool.send("success".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut received = 0;
    let mut results = Vec::new();
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            results.push(res);
            received += 1;
        } else {
            break;
        }
    }

    // Should receive 2 results
    assert_eq!(received, 2);

    Ok(())
}
