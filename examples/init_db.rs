// Initialize database with schema
// Run with: cargo run --example init_db <db_path>

use vscode_agent_monitor::storage_manager::StorageManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <db_path>", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];

    // Creating StorageManager automatically initializes the database with schema
    let _storage = StorageManager::new(db_path)?;

    println!("✅ Database initialized: {}", db_path);

    Ok(())
}
