use anyhow::Result;
use ichika::pool::ThreadPool;

fn main() -> Result<()> {
    let pool_result = ichika::pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
    
    // Explicitly specify the expected type
    let pool: Result<Box<dyn ThreadPool<Request = String, Response = String>>, anyhow::Error> = 
        unsafe { std::mem::transmute(pool_result) };
    
    Ok(())
}
