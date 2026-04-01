use anyhow::Result;

use ichika::pipe;

#[test]
fn create_pipe_with_basic_chain() -> Result<()> {
    let _pool = pipe![
        |req: String| -> usize {
            Ok(if req.len() > 10 {
                req.len() - 10
            } else if req.len() > 5 {
                req.len() - 5
            } else if !req.is_empty() {
                req.len()
            } else {
                0
            })
        },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    Ok(())
}

#[test]
fn create_pipe_with_async() -> Result<()> {
    // Test async closures
    let _pool = pipe![
        async |req: String| -> usize {
            // Simulate async work
            tokio::task::yield_now().await;
            Ok(req.len())
        },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    Ok(())
}
