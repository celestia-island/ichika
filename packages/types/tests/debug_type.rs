use anyhow::Result;

#[test]
fn test_debug() -> Result<()> {
    // Single step with different types
    let _ = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) }
    ];

    Ok(())
}
