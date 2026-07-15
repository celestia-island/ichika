use anyhow::Result;
use ichika::pool::ThreadPool;

#[test]
fn test_explicit_type() -> Result<()> {
    // Verify that a multi-step pipeline with different types compiles and works
    let pool = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;
    pool.send("hello".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    loop {
        if let Some(res) = pool.recv()? {
            assert_eq!(res, "5");
            break;
        }
    }
    Ok(())
}
