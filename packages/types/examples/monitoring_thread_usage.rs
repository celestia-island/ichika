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
        stage1: (max_threads: 1) |req: String| -> usize {
            // A small artificial delay makes the input backlog observable,
            // so task_count("stage1") reports a non-zero depth during the burst.
            std::thread::sleep(std::time::Duration::from_millis(30));
            Ok(req.len())
        },
        stage2: |req: usize| -> String { Ok(format!("processed: {}", req)) },
    ]?;

    // Give daemon time to start
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Send a burst of tasks
    info!("Sending burst of tasks...");
    for i in 0..20 {
        let text = format!("message_{}", i);
        pool.send(text)?;
    }

    // Monitor thread usage periodically; sample the backlog *before* each sleep
    // so the first reading catches the just-queued tasks.
    for iteration in 0..5 {
        let thread_usage = pool.thread_usage()?;
        info!("Iteration {}: Thread usage = {}", iteration, thread_usage);

        // Check task count for each stage by its declared name
        for stage in ["stage1", "stage2"] {
            if let Ok(count) = pool.task_count(stage) {
                info!("  Stage {} task count = {}", stage, count);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Receive all responses
    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut received = 0;
    loop {
        let res = pool.recv()?;
        if res.is_some() {
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
