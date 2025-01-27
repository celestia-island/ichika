use anyhow::Result;

use ichika::create_pool;

#[derive(Debug, Clone)]
struct Request {
    a: i32,
}

#[derive(Debug, Clone)]
struct Request2 {
    b: i32,
}

#[derive(Debug, Clone)]
struct Response {
    c: i32,
}

fn main() -> Result<()> {
    let res = create_pool(move |req: Request| Ok(Request2 { b: req.a + 1 }))
        .pipe(move |req: Request2| Ok(Response { c: req.b + 1 }))
        .run(Request { a: 1 })?;
    println!("Final result: {:?}", res);

    Ok(())
}
