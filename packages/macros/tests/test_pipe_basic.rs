use ichika_macros::pipe;

#[test]
fn test_two_closures_different_types() {
    let _ = pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ];
}
