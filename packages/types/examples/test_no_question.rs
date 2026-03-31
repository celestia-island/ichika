use anyhow::Result;

fn main() -> Result<()> {
    let pool_result = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
    
    match pool_result {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}
