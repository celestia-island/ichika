use ichika::pipe;

fn main() {
    let _pool = pipe![
        |req: String| { Ok(req.len()) },
        |req: usize| { Ok(req.to_string()) }
    ];
}
