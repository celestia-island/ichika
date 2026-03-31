use anyhow::Result;

#[test]
fn test_binding_type() -> Result<()> {
    // Try without underscore
    let pool = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;
    Ok(())
}
