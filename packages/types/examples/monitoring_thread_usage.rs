use anyhow::Result;
use ichika::pipe;
use ichika::pool::ThreadPool;
use log::info;

fn main() -> Result<()> {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .init();

    info!("Starting thread usage monitoring example");

    let pool = pipe![
        |req: String| -> usize {
            Ok(req.len())
        },
        |req: usize| -> String {
            Ok(format!("processed: {}", req))
        },
    ]?;

    // Give daemon time to start
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Send a burst of tasks
    info!("Sending burst of tasks...");
    for i in 0..20 {
        let text = format!("message_{}", i);
        pool.send(text)?;
    }

    // Monitor thread usage periodically
    for iteration in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(100));

        let thread_usage = pool.thread_usage()?;
        info!("Iteration {}: Thread usage = {}", iteration, thread_usage);

        // Check task count for each stage
        // Note: stage names correspond to closure positions
        for stage in ["0", "1"] {
            if let Ok(count) = pool.task_count(stage) {
                info!("  Stage {} task count = {}", stage, count);
            }
        }
    }

    // Receive all responses
    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if let Some(_) = res {
            received += 1;
        } else {
            break;
        }
    }

    info!("Total responses received: {}", received);

    // Final metrics after processing
    std::thread::sleep(std::time::Duration::from_millis(200));
    let final_usage = pool.thread_usage()?;
    info!("Final thread usage = {}", final_usage);

    Ok(())
}
