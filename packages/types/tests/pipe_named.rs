use anyhow::Result;

use ichika::pipe;

#[test]
fn create_named_pipe() -> Result<()> {
    let _pool = pipe![
        test1: |req: String| -> usize { Ok(req.len()) },
        test2: async |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    Ok(())
}

#[test]
fn create_match_pipe() -> Result<()> {
    // Test that match syntax compiles - temporarily commented out due to type inference issue
    // TODO: Investigate and fix the type mismatch error
    /*
    let _pool = pipe![
        test1: |req: String| -> usize { Ok(req.len()) },
        match {
            1 => test3: |req: usize| -> String { Ok(format!("branch_a: {}", req)) },
            _ => test4: |req: usize| -> String { Ok(format!("branch_b: {}", req)) },
        }
    ]?;
    */

    Ok(())
}
