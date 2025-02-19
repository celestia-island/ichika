use anyhow::Result;

use ichika::pipe;

#[test]
fn create_async_pipe() -> Result<()> {
    let pool = pipe![
        async |req: String| -> usize { Ok(req.len()) },
        async |req: usize| -> String { Ok(req.to_string()) }
    ];

    Ok(())
}
