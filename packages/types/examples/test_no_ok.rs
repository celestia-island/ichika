use ichika::pipe;

fn main() {
    let _pool = pipe![
        |req: String| -> usize { req.len() },
        |req: usize| -> String { req.to_string() }
    ];
}
