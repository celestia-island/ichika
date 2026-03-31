// Try with same types for both closures
use anyhow::Result;

fn main() -> Result<()> {
    let pool = ichika::pipe![
        |req: String| -> String { Ok(req.to_uppercase()) },
        |req: String| -> String { Ok(req.to_lowercase()) }
    ]?;
    Ok(())
}
