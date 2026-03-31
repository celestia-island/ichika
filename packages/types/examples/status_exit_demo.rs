use anyhow::Result;
use ichika::pipe;
use ichika::pool::ThreadPool;
use log::info;

fn main() -> Result<()> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    info!("Starting basic pipeline demonstration");

    // Simple two-step pipeline
    let pool = pipe![
        step1: |req: String| -> String {
            info!("Step 1: Processing {}", req);
            // Demonstrate filtering by returning empty for certain inputs
            if req.is_empty() || req.starts_with("skip:") {
                info!("  -> Filtered out");
                Ok("".to_string())
            } else {
                Ok(req.clone())
            }
        },
        step2: |req: String| -> String {
            // Empty strings won't produce meaningful output
            if req.is_empty() {
                Ok("".to_string())
            } else {
                info!("Step 2: Processing {}", req);
                Ok(format!("processed: {}", req))
            }
        },
    ]?;

    // Give daemon time to start
    std::thread::sleep(std::time::Duration::from_millis(200));

    let test_inputs = vec![
        "hello".to_string(),
        "".to_string(),
        "skip:this".to_string(),
        "world".to_string(),
    ];

    for input in test_inputs {
        info!("Send: {:?}", input);
        pool.send(input)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Receive responses
    std::thread::sleep(std::time::Duration::from_millis(200));
    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            if !res.is_empty() {
                info!("Receive: {}", res);
                received += 1;
            }
        } else {
            break;
        }
    }

    info!("Total non-empty responses received: {}", received);
    info!("✓ Pipeline working - filtered inputs produce empty output");

    Ok(())
}
