use anyhow::Result;
use ichika::pipe;
use ichika::pool::ThreadPool;
use log::info;

fn main() -> Result<()> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    info!("Starting graceful shutdown example");

    {
        // Create pool in a scope so it drops at the end
        let pool = pipe![
            |req: String| -> usize {
                info!("Processing: {}", req);
                std::thread::sleep(std::time::Duration::from_millis(50));
                Ok(req.len())
            },
            |req: usize| -> String {
                info!("Formatting: {}", req);
                Ok(format!("result: {}", req))
            },
        ]?;

        // Give daemon time to start
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Submit some tasks
        info!("Submitting tasks...");
        for i in 0..5 {
            pool.send(format!("task_{}", i))?;
        }

        info!("Tasks submitted, waiting briefly for processing...");
        std::thread::sleep(std::time::Duration::from_millis(200));

        // Receive some results
        let mut received = 0;
        for _ in 0..3 {
            if let Ok(Some(res)) = pool.recv() {
                info!("Received: {}", res);
                received += 1;
            }
        }
        info!("Received {} results before scope end", received);

        // Pool drops here, triggering graceful shutdown
        info!("Leaving scope - pool will drop and shutdown gracefully...");
    }

    // After pool drops, verify process exits cleanly
    info!("Pool dropped. Process should exit cleanly without hanging.");

    // If we reach here, graceful shutdown worked
    info!("✓ Graceful shutdown successful!");

    Ok(())
}
