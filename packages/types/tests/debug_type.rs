use anyhow::Result;

#[test]
fn test_debug() -> Result<()> {
    // Single step with different types
    let _pool = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) }
    ]?;

    Ok(())
}
