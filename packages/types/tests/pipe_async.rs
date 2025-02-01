use anyhow::Result;

use ichika::pipe;

#[test]
fn create_async_pipe() -> Result<()> {
    let pool = pipe![
        async |req: String| -> Result<usize> { Ok(req.len()) },
        async |req: usize| -> Result<String> { Ok(req.to_string()) }
    ];

    Ok(())
}
