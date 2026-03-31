use anyhow::Result;

fn main() -> Result<()> {
    let pool = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;
    Ok(())
}
