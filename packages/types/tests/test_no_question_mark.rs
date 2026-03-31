use anyhow::Result;

#[test]
fn test_no_question_mark() -> Result<()> {
    // Try without ?
    let pool_result = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
    // Then explicitly unwrap
    let _pool = pool_result?;
    Ok(())
}
