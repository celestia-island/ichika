use anyhow::Result;
use ichika::pool::ThreadPool;

#[test]
fn test_explicit_type() -> Result<()> {
    // Try with explicit type annotation
    let pool: impl ThreadPool<Request = String, Response = String> = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;
    Ok(())
}
