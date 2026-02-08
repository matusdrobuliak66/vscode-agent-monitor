/// Integration tests for VSCode Agent Monitor
/// Tests end-to-end flows: capture -> scan -> store -> query
use chrono::Utc;
use tempfile::TempDir;
use vscode_agent_monitor::security_scanner::{PatternType, Severity};
use vscode_agent_monitor::storage_manager::StorageManager;
use vscode_agent_monitor::telemetry_collector::TelemetryCollector;

#[test]
fn test_integration_end_to_end_no_secrets() {
    println!("[INTEGRATION] test_integration_end_to_end_no_secrets: Testing complete flow with clean data");

    // Step 1: Capture telemetry with no secrets
    let mut collector = TelemetryCollector::new();
    collector.capture(
        "textDocument/completion",
        r#"{"file": "main.rs", "position": {"line": 10}}"#,
        Some(r#"{"completions": ["fn", "struct", "impl"]}"#),
    );

    println!("[INTEGRATION] Captured 1 telemetry event");
    assert_eq!(collector.count(), 1);

    // Step 2: Verify security scanner found no secrets
    let telemetry = collector.get_latest().unwrap();
    println!("[INTEGRATION] Security findings: {}", telemetry.security_findings.len());
    assert!(
        telemetry.security_findings.is_empty(),
        "Clean data should have no security findings"
    );

    // Step 3: Store telemetry in database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let telemetry_id = storage.store_telemetry(telemetry).unwrap();
    println!("[INTEGRATION] Stored telemetry with ID: {}", telemetry_id);

    // Step 4: Query back from database
    let results = storage.query_by_method("textDocument/completion").unwrap();
    assert_eq!(results.len(), 1, "Should retrieve 1 telemetry record");
    assert_eq!(results[0].method, "textDocument/completion");
    assert!(
        results[0].security_findings.is_empty(),
        "Retrieved record should have no findings"
    );

    println!("[INTEGRATION] test_integration_end_to_end_no_secrets: PASS - End-to-end flow verified");
}

#[test]
fn test_integration_end_to_end_with_api_key() {
    println!("[INTEGRATION] test_integration_end_to_end_with_api_key: Testing secret detection in full flow");

    // Step 1: Capture telemetry with API key in request
    let mut collector = TelemetryCollector::new();
    collector.capture(
        "workspace/configuration",
        r#"{"settings": {"openai": {"apiKey": "sk-1234567890abcdef1234567890abcdef"}}}"#,
        None,
    );

    // Step 2: Verify security scanner detected the API key
    let telemetry = collector.get_latest().unwrap();
    println!("[INTEGRATION] Security findings: {}", telemetry.security_findings.len());
    assert!(!telemetry.security_findings.is_empty(), "Should detect API key");

    let api_key_finding = telemetry
        .security_findings
        .iter()
        .find(|f| matches!(f.pattern_type, PatternType::ApiKey));
    assert!(api_key_finding.is_some(), "Should have ApiKey finding");
    assert!(
        matches!(api_key_finding.unwrap().severity, Severity::High),
        "API key should be High severity"
    );

    // Step 3: Store in database
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let telemetry_id = storage.store_telemetry(telemetry).unwrap();
    println!("[INTEGRATION] Stored telemetry with {} findings", telemetry.security_findings.len());

    // Step 4: Query back and verify findings are persisted
    let findings = storage.get_security_findings(telemetry_id).unwrap();
    assert_eq!(findings.len(), telemetry.security_findings.len());
    assert!(matches!(findings[0].pattern_type, PatternType::ApiKey));
    assert!(matches!(findings[0].severity, Severity::High));

    // Step 5: Query by method and verify findings are included
    let results = storage.query_by_method("workspace/configuration").unwrap();
    assert_eq!(results.len(), 1);
    assert!(!results[0].security_findings.is_empty());

    println!("[INTEGRATION] test_integration_end_to_end_with_api_key: PASS - Secret detection in full flow");
}

#[test]
fn test_integration_security_scanner_catches_password_in_response() {
    println!("[INTEGRATION] test_integration_security_scanner_catches_password_in_response: Testing password detection in response");

    // Step 1: Capture with password in response (security leak simulation)
    let mut collector = TelemetryCollector::new();
    collector.capture(
        "debug/variables",
        r#"{"variableName": "config"}"#,
        Some(r#"{"value": {"db_password": "mysecretpass123"}}"#),
    );

    // Step 2: Verify password was detected
    let telemetry = collector.get_latest().unwrap();
    println!("[INTEGRATION] Security findings: {}", telemetry.security_findings.len());

    let password_finding = telemetry
        .security_findings
        .iter()
        .find(|f| matches!(f.pattern_type, PatternType::Password));
    assert!(password_finding.is_some(), "Should detect password in response");

    // Step 3: Store and verify
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    storage.store_telemetry(telemetry).unwrap();

    // Step 4: Query and verify findings persisted
    let results = storage.query_by_method("debug/variables").unwrap();
    assert_eq!(results.len(), 1);
    assert!(
        results[0].security_findings.iter().any(|f| matches!(f.pattern_type, PatternType::Password)),
        "Password finding should be persisted"
    );

    println!("[INTEGRATION] test_integration_security_scanner_catches_password_in_response: PASS");
}

#[test]
fn test_integration_deduplication_with_real_telemetry() {
    println!("[INTEGRATION] test_integration_deduplication_with_real_telemetry: Testing BLAKE3 dedup with real data");

    // Step 1: Capture multiple telemetry events with duplicate content
    let mut collector = TelemetryCollector::new();
    let duplicate_request = r#"{"textDocument": {"uri": "file:///main.rs"}}"#;

    collector.capture("textDocument/didOpen", duplicate_request, None);
    collector.capture("textDocument/didChange", duplicate_request, None);
    collector.capture("textDocument/didSave", duplicate_request, None);

    assert_eq!(collector.count(), 3, "Should have 3 telemetry events");

    // Step 2: Store all telemetry
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    for telemetry in collector.get_all() {
        storage.store_telemetry(telemetry).unwrap();
    }

    // Step 3: Verify deduplication worked
    let telemetry_count = storage.count_telemetry_logs().unwrap();
    let hash_count = storage.count_file_hashes().unwrap();

    println!("[INTEGRATION] Telemetry logs: {}, Unique hashes: {}", telemetry_count, hash_count);
    assert_eq!(telemetry_count, 3, "Should have 3 telemetry records");
    assert_eq!(hash_count, 1, "Should have only 1 unique hash (content deduplicated)");

    println!("[INTEGRATION] test_integration_deduplication_with_real_telemetry: PASS - Deduplication works");
}

#[test]
fn test_integration_query_by_time_with_mixed_findings() {
    println!("[INTEGRATION] test_integration_query_by_time_with_mixed_findings: Testing time-based queries");

    let mut collector = TelemetryCollector::new();

    // Capture multiple events with different security characteristics
    collector.capture("method1", "normal data", None);
    std::thread::sleep(std::time::Duration::from_millis(10));

    collector.capture("method2", r#"api_key = "sk-testkey1234567890""#, None);
    std::thread::sleep(std::time::Duration::from_millis(10));

    collector.capture("method3", "more normal data", None);

    let all_data = collector.get_all();
    let start_time = all_data[0].timestamp;
    let end_time = all_data[2].timestamp;

    // Store all telemetry
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    for telemetry in all_data {
        storage.store_telemetry(telemetry).unwrap();
    }

    // Query by time range
    let results = storage.query_by_time(start_time, end_time).unwrap();
    println!("[INTEGRATION] Query returned {} results", results.len());
    assert_eq!(results.len(), 3, "Should retrieve all 3 telemetry records");

    // Verify security findings are correctly associated
    let with_findings: Vec<_> = results
        .iter()
        .filter(|r| !r.security_findings.is_empty())
        .collect();
    assert_eq!(with_findings.len(), 1, "Should have exactly 1 record with findings");
    assert_eq!(with_findings[0].method, "method2");

    println!("[INTEGRATION] test_integration_query_by_time_with_mixed_findings: PASS");
}

#[test]
fn test_integration_multiple_findings_same_capture() {
    println!("[INTEGRATION] test_integration_multiple_findings_same_capture: Testing multiple secrets in one event");

    // Capture with multiple secrets
    let mut collector = TelemetryCollector::new();
    collector.capture(
        "config/get",
        r#"{
            "api_key": "sk-test1234567890abcdef",
            "password": "supersecret",
            "token": "9fK2mN8pQ7rT5sU1vW3xY6zA4bC0dE"
        }"#,
        None,
    );

    let telemetry = collector.get_latest().unwrap();
    println!("[INTEGRATION] Found {} security findings", telemetry.security_findings.len());
    assert!(telemetry.security_findings.len() >= 3, "Should detect multiple secrets");

    // Verify different pattern types were detected
    let has_api_key = telemetry
        .security_findings
        .iter()
        .any(|f| matches!(f.pattern_type, PatternType::ApiKey));
    let has_password = telemetry
        .security_findings
        .iter()
        .any(|f| matches!(f.pattern_type, PatternType::Password));
    let has_high_entropy = telemetry
        .security_findings
        .iter()
        .any(|f| matches!(f.pattern_type, PatternType::HighEntropy));

    assert!(has_api_key, "Should detect API key");
    assert!(has_password, "Should detect password");
    assert!(has_high_entropy, "Should detect high entropy");

    // Store and retrieve
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let telemetry_id = storage.store_telemetry(telemetry).unwrap();
    let findings = storage.get_security_findings(telemetry_id).unwrap();

    assert_eq!(findings.len(), telemetry.security_findings.len());

    println!("[INTEGRATION] test_integration_multiple_findings_same_capture: PASS");
}

#[test]
fn test_integration_empty_database_queries() {
    println!("[INTEGRATION] test_integration_empty_database_queries: Testing queries on empty database");

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    // Query empty database
    let results = storage.query_by_method("nonexistent").unwrap();
    assert_eq!(results.len(), 0, "Empty database should return empty results");

    let time_results = storage.query_by_time(Utc::now(), Utc::now()).unwrap();
    assert_eq!(time_results.len(), 0, "Empty database time query should return empty results");

    assert_eq!(storage.count_telemetry_logs().unwrap(), 0);
    assert_eq!(storage.count_file_hashes().unwrap(), 0);

    println!("[INTEGRATION] test_integration_empty_database_queries: PASS - Empty queries handled");
}

#[test]
fn test_integration_collector_clear_and_reuse() {
    println!("[INTEGRATION] test_integration_collector_clear_and_reuse: Testing collector reuse after clear");

    let mut collector = TelemetryCollector::new();

    // First batch
    collector.capture("method1", "data1", None);
    collector.capture("method2", "data2", None);
    assert_eq!(collector.count(), 2);

    // Clear and reuse
    collector.clear();
    assert_eq!(collector.count(), 0);
    assert!(collector.is_empty());

    // Second batch with secrets
    collector.capture("method3", r#"password = "secret""#, None);
    assert_eq!(collector.count(), 1);

    let telemetry = collector.get_latest().unwrap();
    assert!(!telemetry.security_findings.is_empty(), "Should detect password in second batch");

    println!("[INTEGRATION] test_integration_collector_clear_and_reuse: PASS - Collector reuse works");
}

#[test]
fn test_integration_large_batch_storage() {
    println!("[INTEGRATION] test_integration_large_batch_storage: Testing batch storage of multiple events");

    let mut collector = TelemetryCollector::new();

    // Capture 50 telemetry events
    for i in 0..50 {
        let method = format!("method_{}", i % 5); // 5 different methods
        let data = format!(r#"{{"event": {}, "data": "test"}}"#, i);
        collector.capture(&method, &data, None);
    }

    assert_eq!(collector.count(), 50);

    // Store all
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    for telemetry in collector.get_all() {
        storage.store_telemetry(telemetry).unwrap();
    }

    // Verify storage
    let count = storage.count_telemetry_logs().unwrap();
    assert_eq!(count, 50, "Should store all 50 telemetry events");

    // Query by one method (should get 10 results)
    let results = storage.query_by_method("method_0").unwrap();
    assert_eq!(results.len(), 10, "Should retrieve 10 events for method_0");

    println!("[INTEGRATION] test_integration_large_batch_storage: PASS - Batch storage works");
}

#[test]
fn test_integration_edge_case_empty_content() {
    println!("[INTEGRATION] test_integration_edge_case_empty_content: Testing empty content handling");

    let mut collector = TelemetryCollector::new();
    collector.capture("empty_method", "", None);

    let telemetry = collector.get_latest().unwrap();
    assert_eq!(telemetry.request_data, "");
    assert!(telemetry.security_findings.is_empty(), "Empty content should have no findings");

    // Store empty content
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    let telemetry_id = storage.store_telemetry(telemetry).unwrap();
    assert!(telemetry_id > 0, "Should store even empty content");

    let results = storage.query_by_method("empty_method").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].request_data, "");

    println!("[INTEGRATION] test_integration_edge_case_empty_content: PASS - Empty content handled");
}

#[test]
fn test_integration_edge_case_very_long_content() {
    println!("[INTEGRATION] test_integration_edge_case_very_long_content: Testing very long content");

    let mut collector = TelemetryCollector::new();

    // Create a very long string (10KB)
    let long_data = "x".repeat(10_000);
    collector.capture("long_method", &long_data, None);

    let telemetry = collector.get_latest().unwrap();
    assert_eq!(telemetry.request_data.len(), 10_000);

    // Store long content
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    storage.store_telemetry(telemetry).unwrap();

    // Retrieve and verify
    let results = storage.query_by_method("long_method").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].request_data.len(), 10_000);

    println!("[INTEGRATION] test_integration_edge_case_very_long_content: PASS - Long content handled");
}

#[test]
fn test_integration_edge_case_special_characters() {
    println!("[INTEGRATION] test_integration_edge_case_special_characters: Testing special characters");

    let mut collector = TelemetryCollector::new();

    // Content with special characters, quotes, newlines
    let special_data = r#"{"field": "value with \"quotes\"", "newline": "line1\nline2", "unicode": "🔒🔑"}"#;
    collector.capture("special_method", special_data, None);

    let telemetry = collector.get_latest().unwrap();

    // Store special content
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let storage = StorageManager::new(db_path.to_str().unwrap()).unwrap();

    storage.store_telemetry(telemetry).unwrap();

    // Retrieve and verify content is preserved
    let results = storage.query_by_method("special_method").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].request_data, special_data, "Special characters should be preserved");

    println!("[INTEGRATION] test_integration_edge_case_special_characters: PASS - Special chars preserved");
}
