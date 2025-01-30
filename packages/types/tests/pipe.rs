use anyhow::Result;

use ichika::pipe;

#[test]
fn create_pipe() -> Result<()> {
    let pool = pipe![
        |req: String| -> Result<usize> { Ok(req.len()) },
        |req: usize| -> Result<String> { Ok(req.to_string()) }
    ];

    Ok(())
}
