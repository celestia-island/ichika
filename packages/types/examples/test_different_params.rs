use ichika::pipe;

fn main() {
    let _pool = pipe![
        |req1: String| -> usize { Ok(req1.len()) },
        |req2: usize| -> String { Ok(req2.to_string()) }
    ];
}
