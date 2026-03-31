use anyhow::Result;

#[test]
fn test_single_step() -> Result<()> {
    // Single step should work
    let _pool = ichika::pipe![
        |req: String| -> String { Ok(req.to_uppercase()) }
    ]?;
    Ok(())
}
