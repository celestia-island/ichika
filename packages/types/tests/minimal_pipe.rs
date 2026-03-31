use anyhow::Result;

#[test]
fn test_minimal() -> Result<()> {
    // Simplest possible multi-step pipeline
    let _pool = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
    Ok(())
}
