use anyhow::Result;

#[test]
fn test_parse() -> Result<()> {
    // Try with same types
    let _ = ichika::pipe![
        |req: String| -> String { Ok(req.to_uppercase()) }
    ];
    Ok(())
}
