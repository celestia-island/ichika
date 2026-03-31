use anyhow::Result;
use ichika::pool::ThreadPool;

#[test]
fn test_explicit_annotation() -> Result<()> {
    // Try with explicit type annotation on the variable
    let pool_result: Result<impl ThreadPool<Request = String, Response = String>, anyhow::Error> = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
    let _pool = pool_result?;
    Ok(())
}
