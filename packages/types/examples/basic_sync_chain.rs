/// Basic synchronous pipeline example
///
/// Demonstrates a minimal sync pipeline with no branch/error handling.
/// Flow: String -> usize -> String
///
/// Validation: send N requests, collect outputs, assert deterministic mapping

use anyhow::Result;
use ichika::prelude::*;

fn main() -> Result<()> {
    // Initialize logger for demonstration
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("Creating basic sync pipeline: String -> usize -> String");

    // Create a simple 2-step synchronous pipeline
    let pool = pipe![
        |req: String| -> usize {
            log::info!("Step 1: Converting string '{}' to length", req);
            Ok(req.len())
        },
        |req: usize| -> String {
            log::info!("Step 2: Converting length {} back to string", req);
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

    // Collect outputs
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
    log::info!("Demonstration complete: deterministic String -> usize -> String pipeline verified");

    Ok(())
}
