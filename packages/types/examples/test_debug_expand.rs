use ichika::pipe;

fn main() {
    let _ = pipe![
        |req: String| -> String { Ok(req.to_uppercase()) },
        |req: String| -> String { Ok(req.to_lowercase()) }
    ];
}
