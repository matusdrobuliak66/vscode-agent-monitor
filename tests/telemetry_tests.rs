use vscode_agent_monitor::telemetry_collector::{TelemetryCollector, TelemetryData};
use vscode_agent_monitor::security_scanner::{SecurityFinding, Severity, PatternType};
use chrono::Utc;

#[test]
fn test_telemetry_collector_creation() {
    println!("[TEST] test_telemetry_collector_creation: Creating TelemetryCollector");
    let collector = TelemetryCollector::new();
    println!("[TEST] test_telemetry_collector_creation: PASS - Collector created successfully");
    assert!(collector.is_empty());
}

#[test]
fn test_capture_lsp_method_call() {
    println!("[TEST] test_capture_lsp_method_call: Capturing LSP method call");
    let mut collector = TelemetryCollector::new();

    let method = "textDocument/completion";
    let request_data = r#"{"textDocument":{"uri":"file:///test.rs"},"position":{"line":10,"character":5}}"#;

    collector.capture(method, request_data, None);

    println!("[TEST] test_capture_lsp_method_call: Verifying captured data");
    assert_eq!(collector.count(), 1);

    let data = collector.get_latest().unwrap();
    assert_eq!(data.method, method);
    assert_eq!(data.request_data, request_data);
    assert!(data.response_data.is_none());
    println!("[TEST] test_capture_lsp_method_call: PASS - Method call captured correctly");
}

#[test]
fn test_capture_with_response() {
    println!("[TEST] test_capture_with_response: Capturing LSP call with response");
    let mut collector = TelemetryCollector::new();

    let method = "textDocument/hover";
    let request_data = r#"{"textDocument":{"uri":"file:///test.rs"}}"#;
    let response_data = r#"{"contents":"Function documentation"}"#;

    collector.capture(method, request_data, Some(response_data));

    println!("[TEST] test_capture_with_response: Verifying response data");
    let data = collector.get_latest().unwrap();
    assert_eq!(data.method, method);
    assert_eq!(data.response_data, Some(response_data.to_string()));
    println!("[TEST] test_capture_with_response: PASS - Response captured correctly");
}

#[test]
fn test_timestamp_captured() {
    println!("[TEST] test_timestamp_captured: Verifying timestamp capture");
    let mut collector = TelemetryCollector::new();

    let before = Utc::now();
    std::thread::sleep(std::time::Duration::from_millis(10));

    collector.capture("test/method", "request", None);

    std::thread::sleep(std::time::Duration::from_millis(10));
    let after = Utc::now();

    let data = collector.get_latest().unwrap();
    println!("[TEST] test_timestamp_captured: Checking timestamp is between before and after");
    assert!(data.timestamp >= before && data.timestamp <= after);
    println!("[TEST] test_timestamp_captured: PASS - Timestamp captured correctly");
}

#[test]
fn test_multiple_captures() {
    println!("[TEST] test_multiple_captures: Capturing multiple LSP calls");
    let mut collector = TelemetryCollector::new();

    collector.capture("method1", "request1", None);
    collector.capture("method2", "request2", Some("response2"));
    collector.capture("method3", "request3", None);

    println!("[TEST] test_multiple_captures: Verifying count");
    assert_eq!(collector.count(), 3);

    let all_data = collector.get_all();
    assert_eq!(all_data.len(), 3);
    assert_eq!(all_data[0].method, "method1");
    assert_eq!(all_data[1].method, "method2");
    assert_eq!(all_data[2].method, "method3");
    println!("[TEST] test_multiple_captures: PASS - Multiple captures recorded correctly");
}

#[test]
fn test_security_scanner_integration_no_secrets() {
    println!("[TEST] test_security_scanner_integration_no_secrets: Testing with clean data");
    let mut collector = TelemetryCollector::new();

    let clean_request = r#"{"textDocument":{"uri":"file:///test.rs"},"content":"let x = 42;"}"#;
    collector.capture("test/method", clean_request, None);

    let data = collector.get_latest().unwrap();
    println!("[TEST] test_security_scanner_integration_no_secrets: Verifying no security findings");
    assert!(data.security_findings.is_empty());
    println!("[TEST] test_security_scanner_integration_no_secrets: PASS - No false positives");
}

#[test]
fn test_security_scanner_integration_detects_api_key() {
    println!("[TEST] test_security_scanner_integration_detects_api_key: Testing API key detection");
    let mut collector = TelemetryCollector::new();

    let secret_request = r#"{"config":"api_key=sk-1234567890abcdef"}"#;
    collector.capture("test/method", secret_request, None);

    let data = collector.get_latest().unwrap();
    println!("[TEST] test_security_scanner_integration_detects_api_key: Verifying security findings");
    assert!(!data.security_findings.is_empty());

    let finding = &data.security_findings[0];
    assert_eq!(finding.severity, Severity::High);
    assert_eq!(finding.pattern_type, PatternType::ApiKey);
    println!("[TEST] test_security_scanner_integration_detects_api_key: PASS - API key detected");
}

#[test]
fn test_security_scanner_integration_detects_password() {
    println!("[TEST] test_security_scanner_integration_detects_password: Testing password detection");
    let mut collector = TelemetryCollector::new();

    let secret_request = r#"{"credentials":"password=supersecret123"}"#;
    collector.capture("test/method", secret_request, None);

    let data = collector.get_latest().unwrap();
    println!("[TEST] test_security_scanner_integration_detects_password: Verifying password finding");
    assert!(!data.security_findings.is_empty());

    let has_password = data.security_findings.iter().any(|f| {
        matches!(f.pattern_type, PatternType::Password) && matches!(f.severity, Severity::High)
    });
    assert!(has_password);
    println!("[TEST] test_security_scanner_integration_detects_password: PASS - Password detected");
}

#[test]
fn test_security_scanner_scans_response_data() {
    println!("[TEST] test_security_scanner_scans_response_data: Testing response scanning");
    let mut collector = TelemetryCollector::new();

    let clean_request = r#"{"query":"hello"}"#;
    let secret_response = r#"{"result":"Your api_key is sk-abcdef1234567890"}"#;

    collector.capture("test/method", clean_request, Some(secret_response));

    let data = collector.get_latest().unwrap();
    println!("[TEST] test_security_scanner_scans_response_data: Verifying response was scanned");
    assert!(!data.security_findings.is_empty());

    let has_api_key = data.security_findings.iter().any(|f| {
        matches!(f.pattern_type, PatternType::ApiKey)
    });
    assert!(has_api_key);
    println!("[TEST] test_security_scanner_scans_response_data: PASS - Response scanned for secrets");
}

#[test]
fn test_clear_telemetry() {
    println!("[TEST] test_clear_telemetry: Testing clear functionality");
    let mut collector = TelemetryCollector::new();

    collector.capture("method1", "request1", None);
    collector.capture("method2", "request2", None);
    assert_eq!(collector.count(), 2);

    collector.clear();
    println!("[TEST] test_clear_telemetry: Verifying collector is empty after clear");
    assert_eq!(collector.count(), 0);
    assert!(collector.is_empty());
    println!("[TEST] test_clear_telemetry: PASS - Collector cleared successfully");
}

#[test]
fn test_telemetry_data_structure() {
    println!("[TEST] test_telemetry_data_structure: Verifying TelemetryData structure");
    let mut collector = TelemetryCollector::new();

    let method = "textDocument/didOpen";
    let request = r#"{"uri":"file:///test.rs","text":"fn main() {}"}"#;
    let response = r#"{"success":true}"#;

    collector.capture(method, request, Some(response));

    let data = collector.get_latest().unwrap();

    // Verify all fields are accessible
    println!("[TEST] test_telemetry_data_structure: Checking timestamp field");
    let _ = data.timestamp;

    println!("[TEST] test_telemetry_data_structure: Checking method field");
    assert_eq!(data.method, method);

    println!("[TEST] test_telemetry_data_structure: Checking request_data field");
    assert_eq!(data.request_data, request);

    println!("[TEST] test_telemetry_data_structure: Checking response_data field");
    assert_eq!(data.response_data, Some(response.to_string()));

    println!("[TEST] test_telemetry_data_structure: Checking security_findings field");
    let _ = &data.security_findings;

    println!("[TEST] test_telemetry_data_structure: PASS - All fields accessible");
}
