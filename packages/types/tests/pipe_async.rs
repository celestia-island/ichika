use anyhow::Result;

use ichika::pipe;

#[test]
fn create_async_pipe() -> Result<()> {
    let pool = pipe! [
      async |(name: String, checksum: Vec<u8>, url: String)|  {
        Ok((name, id, reqwest::get(url).await?))
      },
      |(name, checksum, buffer)| {
        let mut sha3 = sha3::Sha3_256::new();
        sha3.update(&buffer);
        ensure!(sha3[..] == checksum, "oops!");
        Ok((name, buffer))
      },
      |(name, buffer)| {
        let mut decoder = flate2::read::GzDecoder::new();
        let mut ret = vec![];
        decoder.read_to_end(&mut ret)?;
        Ok((name, data))
      },
      async |(name, data)| {
        tokio::fs::write(
          format!("./{name}.dat"),
          &data
        );
        Ok(())
      }
    ]?;

    Ok(())
}
