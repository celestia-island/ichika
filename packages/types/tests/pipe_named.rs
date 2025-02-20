use anyhow::Result;

use ichika::pipe;

#[test]
fn create_named_pipe() -> Result<()> {
    let pool = pipe![
        test1: |req: String| -> usize { Ok(req.len()) },
        match {
            1 => test3: |req: usize| -> String { Ok(req.to_string()) },
            _ => test4: |req: usize| -> String { Ok(req.to_string()) },
        }
        test2: async |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    Ok(())
}
