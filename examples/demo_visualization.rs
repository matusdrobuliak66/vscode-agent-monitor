// Demo program to test the visualization panel
// Run with: cargo run --example demo_visualization

use vscode_agent_monitor::storage_manager::StorageManager;
use vscode_agent_monitor::telemetry_collector::TelemetryCollector;
use vscode_agent_monitor::visualization_panel::VisualizationPanel;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 VSCode Agent Monitor - Visualization Panel Demo\n");

    // Step 1: Create telemetry collector and capture some sample LSP interactions
    println!("📊 Step 1: Capturing sample telemetry data...");
    let mut collector = TelemetryCollector::new();

    // Sample 1: Completion request with API key (security issue!)
    let request_with_key = r#"{
        "jsonrpc": "2.0",
        "method": "textDocument/completion",
        "params": {
            "apiKey": "sk-1234567890abcdef",
            "context": "const API_KEY = 'sk-proj-1234567890abcdefghijklmnop'"
        }
    }"#;
    collector.capture(
        "textDocument/completion",
        request_with_key,
        Some("{ \"items\": [] }"),
    );
    println!("   ✓ Captured completion request (contains API key)");

    // Sample 2: Definition request with password (security issue!)
    let request_with_password = r#"{
        "jsonrpc": "2.0",
        "method": "textDocument/definition",
        "params": {
            "config": {
                "password": "SuperSecret123!",
                "database": "production"
            }
        }
    }"#;
    collector.capture(
        "textDocument/definition",
        request_with_password,
        Some("{ \"uri\": \"file:///path/to/file.rs\" }"),
    );
    println!("   ✓ Captured definition request (contains password)");

    // Sample 3: Hover request - clean, no secrets
    let clean_request = r#"{
        "jsonrpc": "2.0",
        "method": "textDocument/hover",
        "params": {
            "position": { "line": 10, "character": 5 }
        }
    }"#;
    collector.capture(
        "textDocument/hover",
        clean_request,
        Some("{ \"contents\": \"fn main()\" }"),
    );
    println!("   ✓ Captured hover request (clean, no secrets)");

    // Sample 4: Code action with high-entropy string
    let request_with_entropy = r#"{
        "jsonrpc": "2.0",
        "method": "textDocument/codeAction",
        "params": {
            "token": "aB3dE5fG7hI9jK1lM3nO5pQ7rS9tU1vW3xY5zA7bC9dE1fG3"
        }
    }"#;
    collector.capture(
        "textDocument/codeAction",
        request_with_entropy,
        None,
    );
    println!("   ✓ Captured code action request (high-entropy token)");

    // Sample 5: Multiple secrets in one request
    let request_multi = r#"{
        "jsonrpc": "2.0",
        "method": "textDocument/formatting",
        "params": {
            "api_key": "sk-test-9876543210fedcba",
            "password": "admin123",
            "token": "xY9zA1bC3dE5fG7hI9jK1lM3nO5pQ7rS9tU1vW3"
        }
    }"#;
    collector.capture(
        "textDocument/formatting",
        request_multi,
        Some("{ \"edits\": [] }"),
    );
    println!("   ✓ Captured formatting request (multiple secrets!)\n");

    // Step 2: Store telemetry data in database
    println!("💾 Step 2: Storing telemetry in database...");
    let db_path = "demo_telemetry.db";

    // Remove old database if it exists
    let _ = fs::remove_file(db_path);

    let storage = StorageManager::new(db_path)?;

    for telemetry in collector.get_all() {
        storage.store_telemetry(telemetry)?;
    }

    let count = storage.count_telemetry_logs()?;
    let hash_count = storage.count_file_hashes()?;
    let findings = storage.count_security_findings()?;
    println!("   ✓ Stored {} telemetry events", count);
    println!("   ✓ {} unique content hashes (BLAKE3 deduplication)", hash_count);
    println!("   ✓ Detected {} security findings\n", findings);

    // Step 3: Generate visualization
    println!("🎨 Step 3: Generating HTML visualization...");
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage)?;

    // Save HTML to file
    let output_path = "demo_dashboard.html";
    fs::write(output_path, &html)?;
    println!("   ✓ Generated {} bytes of HTML", html.len());
    println!("   ✓ Saved to: {}\n", output_path);

    // Step 4: Display summary
    println!("✨ Demo Complete!\n");
    println!("📁 Files created:");
    println!("   • {} - SQLite database with telemetry", db_path);
    println!("   • {} - HTML dashboard\n", output_path);
    println!("🌐 To view the dashboard:");
    println!("   Open {} in your web browser\n", output_path);
    println!("Expected findings:");
    println!("   🔴 High: 4 findings (2 API keys, 2 passwords)");
    println!("   🟠 Medium: 2 findings (2 high-entropy strings)");

    Ok(())
}
