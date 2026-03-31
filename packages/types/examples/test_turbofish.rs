use anyhow::Result;

fn main() -> Result<()> {
    // Try with turbofish to help type inference
    let pool: Result<Box<dyn ichika::pool::ThreadPool>, anyhow::Error> = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
    Ok(())
}
