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

#[test]
fn create_pipe_with_multiple_target() -> Result<()> {
    let pool = pipe![
        |req: String| -> Result<usize> { Ok(req.len()) },
        match {
            0 => |req: usize| -> Result<String> { Ok(format!("from 1 {req}")) },
            _ => match {
                1 => |req: usize| -> Result<String> { Ok(format!("from 2 {req}")) },
                _ => |req: usize| -> Result<String> { Ok(format!("from 3 {req}")) }
            },
        },
        |req: usize| -> Result<String> { Ok(req.to_string()) }
    ];

    Ok(())
}

#[test]
fn create_async_pipe() -> Result<()> {
    let pool = pipe![
        async |req: String| -> Result<usize> { Ok(req.len()) },
        async |req: usize| -> Result<String> { Ok(req.to_string()) }
    ];

    Ok(())
}
