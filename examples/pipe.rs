use anyhow::Result;

use ichika::pipe;

fn main() -> Result<()> {
    // let (request, response) = pipe! {
    //     |req: String| { req.len() },
    //     |req: usize| { req.to_string() },
    // };

    // request.send("Hello, World!".to_string())?;
    // let response = response.recv()?;
    // assert_eq!(response, "13");

    Ok(())
}
