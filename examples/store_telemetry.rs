// Store telemetry data from JSON input
// Run with: cargo run --example store_telemetry <db_path> <json_file>

use vscode_agent_monitor::storage_manager::StorageManager;
use vscode_agent_monitor::telemetry_collector::TelemetryCollector;
use std::env;
use std::fs;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <db_path> <json_file>", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    let json_file = &args[2];

    // Read JSON data
    let json_content = fs::read_to_string(json_file)?;
    let data: Value = serde_json::from_str(&json_content)?;

    // Extract fields
    let method = data["method"].as_str().unwrap_or("unknown");
    let request = data["request"].as_str().unwrap_or("{}");
    let response = data["response"].as_str();

    // Create telemetry collector and capture
    let mut collector = TelemetryCollector::new();
    collector.capture(method, request, response);

    // Store to database
    let storage = StorageManager::new(db_path)?;

    for telemetry in collector.get_all() {
        storage.store_telemetry(telemetry)?;
    }

    Ok(())
}
