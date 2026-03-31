// Test to check macro expansion
use ichika::pipe;

fn main() {
    let _ = pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
}
