use anyhow::Result;

fn main() -> Result<()> {
    // Test with single closure
    let _pool1 = ichika::pipe![
        |req: String| -> String { Ok(req.to_uppercase()) }
    ]?;
    
    // Test with two closures same type
    let _pool2 = ichika::pipe![
        |req: String| -> String { Ok(req.to_uppercase()) },
        |req: String| -> String { Ok(req.to_lowercase()) }
    ]?;
    
    Ok(())
}
