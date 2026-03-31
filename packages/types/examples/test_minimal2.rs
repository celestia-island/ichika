use anyhow::Result;

fn main() -> Result<()> {
    let pool: Result<_, _> = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
    let _pool = pool?;
    Ok(())
}
