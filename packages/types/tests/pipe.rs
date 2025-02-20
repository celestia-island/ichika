use anyhow::Result;

use ichika::{pipe, pool::ThreadPool};

#[test]
fn create_pipe() -> Result<()> {
    let pool = pipe![
        |req: String| -> usize { Ok(req.len()) },
        |req: usize| -> String { Ok(req.to_string()) }
    ]?;
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    // Test case
    // Generate some random string with random length
    const TEST_CASE_MAX_COUNT: usize = 10;
    for i in 0..TEST_CASE_MAX_COUNT {
        for j in 0..TEST_CASE_MAX_COUNT {
            let req = (i..TEST_CASE_MAX_COUNT)
                .map(|_| ('a' as u8 + j as u8) as char)
                .collect::<String>();
            log::info!("Send: {:?}", req);
            pool.send(req)?;
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            log::info!("Receive: {:?}", res);
        } else {
            break;
        }
    }
    log::info!("All responses are received");

    Ok(())
}
