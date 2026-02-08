// Query events with filtering
// Run with: cargo run --example query_events <db_path> [--method METHOD] [--start START] [--end END] [--limit LIMIT]

use vscode_agent_monitor::storage_manager::StorageManager;
use std::env;
use serde_json::json;
use rusqlite::Result as SqlResult;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <db_path> [--method METHOD] [--start START] [--end END] [--limit LIMIT]", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    let storage = StorageManager::new(db_path)?;

    // Parse optional filters
    let mut method_filter: Option<String> = None;
    let mut limit: usize = 100;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--method" if i + 1 < args.len() => {
                method_filter = Some(args[i + 1].clone());
                i += 2;
            }
            "--limit" if i + 1 < args.len() => {
                limit = args[i + 1].parse().unwrap_or(100);
                i += 2;
            }
            _ => i += 1,
        }
    }

    // Query events
    let conn = storage.get_connection();

    let query = if let Some(method) = &method_filter {
        format!("
            SELECT t.id, t.timestamp, t.method, t.request_data_hash, t.response_data_hash,
                   req.content as request_content, res.content as response_content,
                   (SELECT COUNT(*) FROM security_findings WHERE telemetry_id = t.id) as finding_count
            FROM telemetry_logs t
            JOIN file_hashes req ON t.request_data_hash = req.id
            LEFT JOIN file_hashes res ON t.response_data_hash = res.id
            WHERE t.method = '{}'
            ORDER BY t.timestamp DESC
            LIMIT {}
        ", method, limit)
    } else {
        format!("
            SELECT t.id, t.timestamp, t.method, t.request_data_hash, t.response_data_hash,
                   req.content as request_content, res.content as response_content,
                   (SELECT COUNT(*) FROM security_findings WHERE telemetry_id = t.id) as finding_count
            FROM telemetry_logs t
            JOIN file_hashes req ON t.request_data_hash = req.id
            LEFT JOIN file_hashes res ON t.response_data_hash = res.id
            ORDER BY t.timestamp DESC
            LIMIT {}
        ", limit)
    };

    let mut stmt = conn.prepare(&query)?;
    let events: Vec<_> = stmt.query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "timestamp": row.get::<_, i64>(1)?,
            "method": row.get::<_, String>(2)?,
            "request": row.get::<_, String>(5)?,
            "response": row.get::<_, Option<String>>(6)?,
            "findingCount": row.get::<_, i64>(7)?
        }))
    })?.collect::<Result<Vec<_>, _>>()?;

    // Output as JSON
    println!("{}", serde_json::to_string_pretty(&json!(events))?);

    Ok(())
}
