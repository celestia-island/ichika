use anyhow::Result;
use ichika::pipe;
use ichika::pool::ThreadPool;
use log::info;

fn main() -> Result<()> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    info!("Starting string processing pipeline example");

    // Note: True tuple payload support requires macro enhancement
    // For now, demonstrating string processing chain
    let pool = pipe![
        // Step 1: Transform string
        process_string: |req: String| -> String {
            info!("Step 1: Received {:?}", req);
            Ok(req.to_uppercase())
        },
        // Step 2: Add metadata
        add_metadata: |req: String| -> String {
            info!("Step 2: Processing {}", req);
            let len = req.len();
            Ok(format!("{} (length: {})", req, len))
        },
    ]?;

    // Give daemon time to start
    std::thread::sleep(std::time::Duration::from_millis(200));

    let test_cases = vec![
        "hello",
        "world",
        "test",
    ];

    for text in test_cases {
        info!("Send: {:?}", text);
        pool.send(text.to_string())?;
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Receive all responses
    std::thread::sleep(std::time::Duration::from_millis(200));
    loop {
        let res = pool.recv()?;
        if let Some(res) = res {
            info!("Receive: {}", res);
        } else {
            break;
        }
    }

    info!("All responses received");

    Ok(())
}
