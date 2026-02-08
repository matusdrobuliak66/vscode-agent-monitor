// Get context for a specific security finding
// Run with: cargo run --example get_finding_context <db_path> <telemetry_id>

use vscode_agent_monitor::storage_manager::StorageManager;
use std::env;
use serde_json::json;
use rusqlite::Result as SqlResult;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <db_path> <telemetry_id>", args[0]);
        std::process::exit(1);
    }

    let db_path = &args[1];
    let telemetry_id: i64 = args[2].parse()?;

    let storage = StorageManager::new(db_path)?;
    let conn = storage.get_connection();

    // Get telemetry event with full context
    let query = "
        SELECT t.id, t.timestamp, t.method,
               req.content as request_content,
               res.content as response_content
        FROM telemetry_logs t
        JOIN file_hashes req ON t.request_data_hash = req.id
        LEFT JOIN file_hashes res ON t.response_data_hash = res.id
        WHERE t.id = ?
    ";

    let event = conn.query_row(query, [telemetry_id], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "timestamp": row.get::<_, i64>(1)?,
            "method": row.get::<_, String>(2)?,
            "request": row.get::<_, String>(3)?,
            "response": row.get::<_, Option<String>>(4)?
        }))
    })?;

    // Get security findings for this event
    let findings_query = "
        SELECT line_num, severity, pattern_type
        FROM security_findings
        WHERE telemetry_id = ?
        ORDER BY line_num
    ";

    let mut stmt = conn.prepare(findings_query)?;
    let findings: Vec<_> = stmt.query_map([telemetry_id], |row| {
        Ok(json!({
            "lineNum": row.get::<_, i64>(0)?,
            "severity": row.get::<_, String>(1)?,
            "patternType": row.get::<_, String>(2)?
        }))
    })?.collect::<Result<Vec<_>, _>>()?;

    // Output combined result
    println!("{}", serde_json::to_string_pretty(&json!({
        "event": event,
        "findings": findings
    }))?);

    Ok(())
}
