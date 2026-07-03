use anyhow::Result;
use ichika::pipe;
use ichika::pool::ThreadPool;

#[test]
fn test_basic_named_routing() -> Result<()> {
    // Test that named steps are properly identified and channels are created
    let pool = pipe![
        step1: |req: String| -> usize { Ok(req.len()) },
        step2: |req: usize| -> String { Ok(format!("processed: {}", req)) },
    ]?;

    // Give daemon time to start up
    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("hello".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Loop to receive like the working test does
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, "processed: 5");
            break;
        }
    }

    Ok(())
}

#[test]
fn test_multiple_named_steps() -> Result<()> {
    // Test chain of named steps
    let pool = pipe![
        parser: |req: String| -> usize { Ok(req.len()) },
        doubler: |req: usize| -> usize { Ok(req * 2) },
        formatter: |req: usize| -> String { Ok(format!("final: {}", req)) },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));
    pool.send("test".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, "final: 8"); // "test".len() = 4, *2 = 8
            break;
        }
    }

    Ok(())
}

#[test]
fn test_async_routing() -> Result<()> {
    // Test async step in named pipeline
    let pool = pipe![
        step1: async |req: String| -> usize {
            tokio::task::yield_now().await;
            Ok(req.len())
        },
        step2: |req: usize| -> String { Ok(format!("async result: {}", req)) },
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));
    pool.send("test".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, "async result: 4");
            break;
        }
    }

    Ok(())
}

#[test]
fn test_match_routing_e2e() -> Result<()> {
    // E2E test for match routing - verify that dispatcher correctly routes to branches
    let pool = pipe![
        step1: |req: String| -> usize { Ok(req.len()) },
        match {
            1 => branch1: |req: usize| -> String { Ok(format!("one: {}", req)) },
            _ => branch2: |req: usize| -> String { Ok(format!("other: {}", req)) },
        }
    ]?;

    // Give daemon time to start up
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Test case 1: Send "hello" (length 5) -> should route to branch2 (wildcard)
    pool.send("hello".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, "other: 5");
            break;
        }
    }

    // Test case 2: Send "h" (length 1) -> should route to branch1
    pool.send("h".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, "one: 1");
            break;
        }
    }

    Ok(())
}

#[test]
fn test_match_routing_on_non_copy_type() -> Result<()> {
    // The dispatcher's input here is `String` (Clone, but NOT Copy). Previously
    // the generated dispatcher `match req { .. => Switch((_, req)) }` failed to
    // compile because `req` was moved into the match scrutinee. This locks in
    // the fix and verifies the value is forwarded correctly to the branch.
    let pool = pipe![
        classify: |req: String| -> String { Ok(req) },
        match {
            _ => sink: |req: String| -> usize { Ok(req.len()) },
        }
    ]?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    pool.send("hello".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(300));

    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            assert_eq!(res, 5);
            break;
        }
    }

    Ok(())
}
