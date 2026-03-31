/// Basic asynchronous pipeline example
///
/// Demonstrates an async closure chain under default tokio feature.
/// Flow: async step compute/transform, final output collect
///
/// Validation: successful send/recv loop; no panic on runtime path

use anyhow::Result;
use ichika::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger for demonstration
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Creating basic async pipeline: String -> usize -> String");

    // Create a simple 2-step asynchronous pipeline
    let pool = pipe![
        async |req: String| -> usize {
            log::info!("Step 1: Async converting string '{}' to length", req);
            // Simulate async work with tokio::task::yield_now
            tokio::task::yield_now().await;
            Ok(req.len())
        },
        async |req: usize| -> String {
            log::info!("Step 2: Async converting length {} back to string", req);
            // Simulate async work
            tokio::task::yield_now().await;
            Ok(req.to_string())
        }
    ]?;

    log::info!("Pipeline created successfully");

    // Test inputs
    let inputs = vec![
        "hello".to_string(),
        "world".to_string(),
        "ichika".to_string(),
        "rust".to_string(),
        "pipeline".to_string(),
    ];

    log::info!("Sending {} requests", inputs.len());

    // Send all inputs
    for input in &inputs {
        log::info!("Sending: '{}'", input);
        pool.send(input.clone())?;
    }

    log::info!("All requests sent, collecting outputs...");

    // Expected outputs for validation
    let expected_outputs: Vec<String> = inputs
        .iter()
        .map(|s| s.len().to_string())
        .collect();

    let mut collected_outputs = Vec::new();

    // Collect outputs - note: recv() returns Result<Option<Response>>
    loop {
        match pool.recv()? {
            Some(output) => {
                log::info!("Received: '{}'", output);
                collected_outputs.push(output);
            }
            None => {
                log::info!("No more outputs");
                break;
            }
        }
    }

    log::info!("Collected {} outputs", collected_outputs.len());

    // Validate deterministic mapping
    assert_eq!(
        collected_outputs.len(),
        inputs.len(),
        "Expected {} outputs, got {}",
        inputs.len(),
        collected_outputs.len()
    );

    for (i, (input, expected)) in inputs.iter().zip(expected_outputs.iter()).enumerate() {
        let actual = &collected_outputs[i];
        log::info!("Validation: '{}' -> {} (expected: {})", input, actual, expected);
        assert_eq!(
            actual, expected,
            "Output mismatch for input '{}': expected {}, got {}",
            input, expected, actual
        );
    }

    log::info!("All validations passed!");
    log::info!("Demonstration complete: async String -> usize -> String pipeline verified");

    Ok(())
}
