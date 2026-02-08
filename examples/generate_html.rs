// Generate HTML dashboard and write to file
// Run with: cargo run --example generate_html <db_path> <output_path>

use vscode_agent_monitor::storage_manager::StorageManager;
use vscode_agent_monitor::visualization_panel::VisualizationPanel;
use std::env;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <db_path> <output_path>", args[0]);
        eprintln!("Example: {} demo_telemetry.db dashboard.html", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    let output_path = &args[2];

    // Open storage manager
    let storage = StorageManager::new(db_path)?;

    // Generate HTML
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage)?;

    // Write to file
    fs::write(output_path, html)?;

    println!("✅ Dashboard generated: {}", output_path);

    Ok(())
}
