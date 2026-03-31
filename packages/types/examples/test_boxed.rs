use ichika::pipe;

fn main() {
    let _pool = pipe![
        Box::new(|req: String| -> usize { Ok(req.len()) }),
        Box::new(|req: usize| -> String { Ok(req.to_string()) })
    ];
}
