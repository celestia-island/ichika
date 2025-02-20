use anyhow::Result;

use ichika::pipe;

#[test]
fn create_pipe_with_result_target() -> Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            if req.len() > 10 {
                Ok(1)
            } else {
                Err("error")
            }
        },
        catch |req: &str| -> String { Ok(req.to_string()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;

    Ok(())
}

#[test]
fn create_pipe_with_switchable_target() -> Result<()> {
    let pool = pipe![
        |req: String| -> usize {
            if req.len() > 10 {
                (ichika::Status::Switch("target_a"), req.len() - 10)
            } else if req.len() > 5 {
                (ichika::Status::Switch("target_b"), req.len() - 5)
            } else if req.len() > 0 {
                (ichika::Status::Switch("target_c"), req.len())
            } else {
                ichika::Status::Exit
            }
        },
        match {
            target_a: |req: usize| -> String { Ok(format!("a{req}")) },
            target_b: |req: usize| -> String { Ok(format!("b {req}")) },
            target_c: |req: usize| -> String { Ok(format!("c {req}")) }
        },
        |req: usize| -> String { Ok(req.to_string()) }
    ];

    Ok(())
}
