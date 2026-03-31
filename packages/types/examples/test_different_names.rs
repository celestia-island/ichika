use anyhow::Result;

fn main() -> Result<()> {
    let pool = ichika::pipe![
        |input: String| -> usize { Ok(input.len()) },
        |data: usize| -> String { Ok(data.to_string()) }
    ]?;
    Ok(())
}
