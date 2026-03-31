use ichika::test_identity;

fn main() {
    let _ = test_identity!(
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    );
}
