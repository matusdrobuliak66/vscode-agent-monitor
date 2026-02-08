use chrono::{DateTime, TimeZone, Utc};
use tempfile::TempDir;
use vscode_agent_monitor::security_scanner::{SecurityFinding, Severity, PatternType};
use vscode_agent_monitor::storage_manager::StorageManager;
use vscode_agent_monitor::telemetry_collector::TelemetryData;

/// Helper function to create a test TelemetryData with security findings
fn create_test_telemetry_with_findings(
    timestamp: DateTime<Utc>,
    method: &str,
    request: &str,
    response: Option<&str>,
    findings: Vec<SecurityFinding>,
) -> TelemetryData {
    TelemetryData {
        timestamp,
        method: method.to_string(),
        request_data: request.to_string(),
        response_data: response.map(|s| s.to_string()),
        security_findings: findings,
    }
}

/// Helper function to create a simple test TelemetryData
fn create_test_telemetry(
    timestamp: DateTime<Utc>,
    method: &str,
    request: &str,
    response: Option<&str>,
) -> TelemetryData {
    create_test_telemetry_with_findings(timestamp, method, request, response, vec![])
}

#[test]
fn test_storage_init_db_creates_tables() {
    println!("TEST: Database initialization creates required tables");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    // Verify tables exist by attempting to query them
    let telemetry_count = storage.count_telemetry_logs().unwrap();
    assert_eq!(telemetry_count, 0, "New database should have 0 telemetry logs");

    println!("PASS: Database initialized with all tables");
}

#[test]
fn test_storage_store_simple_telemetry() {
    println!("TEST: Store simple telemetry without response or findings");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();
    let telemetry = create_test_telemetry(
        timestamp,
        "textDocument/completion",
        r#"{"file": "test.rs"}"#,
        None,
    );

    let telemetry_id = storage.store_telemetry(&telemetry).unwrap();
    assert!(telemetry_id > 0, "Should return valid telemetry ID");

    let count = storage.count_telemetry_logs().unwrap();
    assert_eq!(count, 1, "Should have 1 telemetry log stored");

    println!("PASS: Simple telemetry stored successfully");
}

#[test]
fn test_storage_store_telemetry_with_response() {
    println!("TEST: Store telemetry with request and response");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();
    let telemetry = create_test_telemetry(
        timestamp,
        "textDocument/hover",
        r#"{"position": {"line": 10}}"#,
        Some(r#"{"contents": "Function signature"}"#),
    );

    let telemetry_id = storage.store_telemetry(&telemetry).unwrap();
    assert!(telemetry_id > 0, "Should return valid telemetry ID");

    println!("PASS: Telemetry with response stored successfully");
}

#[test]
fn test_storage_file_hash_deduplication() {
    println!("TEST: BLAKE3 hashing deduplicates identical content");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let same_content = r#"{"file": "duplicate.rs"}"#;

    // Store first telemetry with this content
    let timestamp1 = Utc.timestamp_opt(1700000000, 0).unwrap();
    let telemetry1 = create_test_telemetry(
        timestamp1,
        "method1",
        same_content,
        None,
    );
    storage.store_telemetry(&telemetry1).unwrap();

    // Store second telemetry with identical content
    let timestamp2 = Utc.timestamp_opt(1700000100, 0).unwrap();
    let telemetry2 = create_test_telemetry(
        timestamp2,
        "method2",
        same_content,
        None,
    );
    storage.store_telemetry(&telemetry2).unwrap();

    // Verify: 2 telemetry logs but only 1 unique file hash
    let telemetry_count = storage.count_telemetry_logs().unwrap();
    let hash_count = storage.count_file_hashes().unwrap();

    assert_eq!(telemetry_count, 2, "Should have 2 telemetry logs");
    assert_eq!(hash_count, 1, "Should have only 1 file hash (deduplicated)");

    println!("PASS: Content deduplication working correctly");
}

#[test]
fn test_storage_store_security_findings() {
    println!("TEST: Store telemetry with security findings");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();
    let findings = vec![
        SecurityFinding {
            line_num: 5,
            severity: Severity::High,
            pattern_type: PatternType::ApiKey,
        },
        SecurityFinding {
            line_num: 10,
            severity: Severity::Medium,
            pattern_type: PatternType::HighEntropy,
        },
    ];

    let telemetry = create_test_telemetry_with_findings(
        timestamp,
        "textDocument/didOpen",
        "API_KEY=sk-12345678901234567890",
        None,
        findings,
    );

    let telemetry_id = storage.store_telemetry(&telemetry).unwrap();

    // Retrieve security findings
    let retrieved_findings = storage.get_security_findings(telemetry_id).unwrap();
    assert_eq!(retrieved_findings.len(), 2, "Should have 2 security findings");

    // Verify first finding
    assert_eq!(retrieved_findings[0].line_num, 5);
    assert!(matches!(retrieved_findings[0].severity, Severity::High));
    assert!(matches!(retrieved_findings[0].pattern_type, PatternType::ApiKey));

    // Verify second finding
    assert_eq!(retrieved_findings[1].line_num, 10);
    assert!(matches!(retrieved_findings[1].severity, Severity::Medium));
    assert!(matches!(retrieved_findings[1].pattern_type, PatternType::HighEntropy));

    println!("PASS: Security findings stored and retrieved correctly");
}

#[test]
fn test_storage_query_by_time_range() {
    println!("TEST: Query telemetry by timestamp range");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    // Store telemetry at different times
    let t1 = Utc.timestamp_opt(1700000000, 0).unwrap(); // Nov 14, 2023
    let t2 = Utc.timestamp_opt(1700001000, 0).unwrap(); // ~16 minutes later
    let t3 = Utc.timestamp_opt(1700002000, 0).unwrap(); // ~33 minutes later

    storage.store_telemetry(&create_test_telemetry(t1, "method1", "data1", None)).unwrap();
    storage.store_telemetry(&create_test_telemetry(t2, "method2", "data2", None)).unwrap();
    storage.store_telemetry(&create_test_telemetry(t3, "method3", "data3", None)).unwrap();

    // Query for middle period
    let start = Utc.timestamp_opt(1700000500, 0).unwrap();
    let end = Utc.timestamp_opt(1700001500, 0).unwrap();

    let results = storage.query_by_time(start, end).unwrap();
    assert_eq!(results.len(), 1, "Should find exactly 1 telemetry in range");
    assert_eq!(results[0].method, "method2");

    println!("PASS: Time range query returned correct results");
}

#[test]
fn test_storage_query_by_method() {
    println!("TEST: Query telemetry by LSP method");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();

    // Store multiple telemetry records with different methods
    storage.store_telemetry(&create_test_telemetry(
        timestamp,
        "textDocument/completion",
        "data1",
        None,
    )).unwrap();
    storage.store_telemetry(&create_test_telemetry(
        timestamp,
        "textDocument/hover",
        "data2",
        None,
    )).unwrap();
    storage.store_telemetry(&create_test_telemetry(
        timestamp,
        "textDocument/completion",
        "data3",
        None,
    )).unwrap();

    // Query for completion method
    let results = storage.query_by_method("textDocument/completion").unwrap();
    assert_eq!(results.len(), 2, "Should find 2 completion telemetry records");

    // Query for hover method
    let hover_results = storage.query_by_method("textDocument/hover").unwrap();
    assert_eq!(hover_results.len(), 1, "Should find 1 hover telemetry record");

    println!("PASS: Method query returned correct results");
}

#[test]
fn test_storage_query_empty_results() {
    println!("TEST: Queries return empty vectors when no matches");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();
    storage.store_telemetry(&create_test_telemetry(
        timestamp,
        "textDocument/completion",
        "data",
        None,
    )).unwrap();

    // Query for non-existent method
    let results = storage.query_by_method("nonexistent/method").unwrap();
    assert_eq!(results.len(), 0, "Should return empty vector for no matches");

    // Query for out-of-range time
    let start = Utc.timestamp_opt(1600000000, 0).unwrap();
    let end = Utc.timestamp_opt(1600001000, 0).unwrap();
    let time_results = storage.query_by_time(start, end).unwrap();
    assert_eq!(time_results.len(), 0, "Should return empty vector for time range with no data");

    println!("PASS: Empty queries handled correctly");
}

#[test]
fn test_storage_retrieve_with_security_findings() {
    println!("TEST: Retrieved telemetry includes associated security findings");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();
    let findings = vec![
        SecurityFinding {
            line_num: 3,
            severity: Severity::High,
            pattern_type: PatternType::Password,
        },
    ];

    let telemetry = create_test_telemetry_with_findings(
        timestamp,
        "textDocument/didSave",
        "password=secret123",
        None,
        findings,
    );

    let _telemetry_id = storage.store_telemetry(&telemetry).unwrap();

    // Query by method and check findings are included
    let results = storage.query_by_method("textDocument/didSave").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].security_findings.len(), 1);
    assert_eq!(results[0].security_findings[0].line_num, 3);

    println!("PASS: Telemetry queries include security findings");
}

#[test]
fn test_storage_multiple_findings_per_telemetry() {
    println!("TEST: Store and retrieve multiple security findings per telemetry");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();
    let findings = vec![
        SecurityFinding {
            line_num: 1,
            severity: Severity::High,
            pattern_type: PatternType::ApiKey,
        },
        SecurityFinding {
            line_num: 2,
            severity: Severity::High,
            pattern_type: PatternType::Password,
        },
        SecurityFinding {
            line_num: 3,
            severity: Severity::Medium,
            pattern_type: PatternType::HighEntropy,
        },
    ];

    let telemetry = create_test_telemetry_with_findings(
        timestamp,
        "test_method",
        "test data",
        None,
        findings,
    );

    let telemetry_id = storage.store_telemetry(&telemetry).unwrap();
    let retrieved = storage.get_security_findings(telemetry_id).unwrap();

    assert_eq!(retrieved.len(), 3, "Should retrieve all 3 findings");
    assert_eq!(retrieved[0].line_num, 1);
    assert_eq!(retrieved[1].line_num, 2);
    assert_eq!(retrieved[2].line_num, 3);

    println!("PASS: Multiple findings per telemetry handled correctly");
}

#[test]
fn test_storage_blake3_hash_consistency() {
    println!("TEST: BLAKE3 hashes are consistent for identical content");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let content = "consistent test data";

    // Store same content in two different telemetry records
    let t1 = Utc.timestamp_opt(1700000000, 0).unwrap();
    let t2 = Utc.timestamp_opt(1700001000, 0).unwrap();

    storage.store_telemetry(&create_test_telemetry(t1, "m1", content, None)).unwrap();
    storage.store_telemetry(&create_test_telemetry(t2, "m2", content, None)).unwrap();

    // Both should reference the same hash
    let hash_count = storage.count_file_hashes().unwrap();
    assert_eq!(hash_count, 1, "Identical content should produce same hash");

    println!("PASS: BLAKE3 hash consistency verified");
}

#[test]
fn test_storage_persistence_across_instances() {
    println!("TEST: Data persists when reopening database");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Store data with first instance
    {
        let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();
        let timestamp = Utc.timestamp_opt(1700000000, 0).unwrap();
        storage.store_telemetry(&create_test_telemetry(
            timestamp,
            "test_method",
            "persistent data",
            None,
        )).unwrap();
    }

    // Reopen and verify data exists
    {
        let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();
        let count = storage.count_telemetry_logs().unwrap();
        assert_eq!(count, 1, "Data should persist after closing and reopening");

        let results = storage.query_by_method("test_method").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].request_data, "persistent data");
    }

    println!("PASS: Database persistence verified");
}
