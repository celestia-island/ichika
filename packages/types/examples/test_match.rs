use anyhow::Result;

fn main() -> Result<()> {
    let pipe_result = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
    
    match pipe_result {
        Ok(pool) => {
            println!("Pool created");
            Ok(())
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    }
}
