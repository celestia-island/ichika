use anyhow::Result;

use ichika::pipe;

#[test]
fn create_named_pipe() -> Result<()> {
    let pool = pipe![
        test1: |req: String| -> Result<usize> { Ok(req.len()) },
        match {
            1 => test3: |req: usize| -> Result<String> { Ok(req.to_string()) },
            _ => test4: |req: usize| -> Result<String> { Ok(req.to_string()) },
        }
        test2: async |req: usize| -> Result<String> { Ok(req.to_string()) }
    ];

    Ok(())
}
