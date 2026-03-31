use anyhow::Result;

fn main() -> Result<()> {
    // Single closure - same input and output type
    let pool = ichika::pipe![
        |req: String| -> String { Ok(req.to_uppercase()) }
    ]?;
    Ok(())
}
