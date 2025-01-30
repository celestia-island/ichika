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
        |req: String| -> (Result<usize>, Channel) { (Ok(req.len()), if req.len() > 3 { 0 } else { 1 }) },
        [
            |req: usize| -> Result<String> { Ok(format!("from 1 {req}")) },
            |req: usize| -> Result<String> { Ok(format!("from 2 {req}")) }
        ]
        |req: usize| -> Result<String> { Ok(req.to_string()) }
    ];

    Ok(())
}
