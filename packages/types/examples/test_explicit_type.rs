use ichika::pipe;

fn main() {
    let _pool: Result<(), anyhow::Error> = pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
}
