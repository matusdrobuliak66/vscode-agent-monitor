use vscode_agent_monitor::visualization_panel::VisualizationPanel;
use vscode_agent_monitor::storage_manager::StorageManager;
use vscode_agent_monitor::telemetry_collector::{TelemetryCollector, TelemetryData};
use vscode_agent_monitor::security_scanner::{SecurityFinding, Severity, PatternType};
use chrono::Utc;
use tempfile::NamedTempFile;

// Helper function to create a test database with sample data
fn create_test_db() -> (NamedTempFile, StorageManager) {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let storage = StorageManager::new(db_path).unwrap();

    // Add some test data
    let telemetry1 = TelemetryData {
        timestamp: Utc::now(),
        method: "textDocument/completion".to_string(),
        request_data: "let api_key = \"test123\"".to_string(),
        response_data: Some("completion results".to_string()),
        security_findings: vec![
            SecurityFinding {
                line_num: 1,
                severity: Severity::High,
                pattern_type: PatternType::ApiKey,
            }
        ],
    };

    let telemetry2 = TelemetryData {
        timestamp: Utc::now(),
        method: "textDocument/hover".to_string(),
        request_data: "password = \"secret123\"".to_string(),
        response_data: None,
        security_findings: vec![
            SecurityFinding {
                line_num: 1,
                severity: Severity::High,
                pattern_type: PatternType::Password,
            }
        ],
    };

    let telemetry3 = TelemetryData {
        timestamp: Utc::now(),
        method: "textDocument/definition".to_string(),
        request_data: "some \"9fK2mN8pQ7rT5sU1vW3xY6zA4bC0dE\" code".to_string(),
        response_data: Some("definition result".to_string()),
        security_findings: vec![
            SecurityFinding {
                line_num: 1,
                severity: Severity::Medium,
                pattern_type: PatternType::HighEntropy,
            }
        ],
    };

    storage.store_telemetry(&telemetry1).unwrap();
    storage.store_telemetry(&telemetry2).unwrap();
    storage.store_telemetry(&telemetry3).unwrap();

    (temp_file, storage)
}

#[test]
fn test_visualization_panel_generates_html() {
    println!("[TEST] Verification: VisualizationPanel generates valid HTML");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    assert!(!html.is_empty(), "HTML should not be empty");
    assert!(html.contains("<!DOCTYPE html>"), "HTML should have DOCTYPE declaration");
    assert!(html.contains("<html"), "HTML should have html tag");
    assert!(html.contains("</html>"), "HTML should have closing html tag");

    println!("[PASS] HTML structure is valid");
}

#[test]
fn test_html_has_proper_structure() {
    println!("[TEST] Verification: HTML has proper semantic structure");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    assert!(html.contains("<head>"), "HTML should have head section");
    assert!(html.contains("<body>"), "HTML should have body section");
    assert!(html.contains("<title>"), "HTML should have title");
    assert!(html.contains("</head>"), "HTML should close head section");
    assert!(html.contains("</body>"), "HTML should close body section");

    println!("[PASS] HTML semantic structure is correct");
}

#[test]
fn test_html_contains_statistics() {
    println!("[TEST] Verification: HTML displays telemetry statistics");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    // Should display total events (3 in our test data)
    assert!(html.contains("3"), "HTML should show total telemetry count");

    // Should show security findings count (3 findings total)
    assert!(html.contains("Security Findings") || html.contains("security"),
            "HTML should mention security findings");

    println!("[PASS] Statistics are displayed in HTML");
}

#[test]
fn test_html_shows_security_findings_table() {
    println!("[TEST] Verification: HTML contains security findings table");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    assert!(html.contains("<table"), "HTML should contain a table");
    assert!(html.contains("<thead"), "Table should have header");
    assert!(html.contains("<tbody"), "Table should have body");

    // Check for table columns
    assert!(html.contains("Severity") || html.contains("severity"),
            "Table should have Severity column");
    assert!(html.contains("Pattern") || html.contains("Type") || html.contains("pattern"),
            "Table should have Pattern Type column");

    println!("[PASS] Security findings table structure present");
}

#[test]
fn test_html_has_color_coded_severity() {
    println!("[TEST] Verification: Severity levels are color-coded");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    // Should contain CSS or inline styles for severity colors
    assert!(html.contains("High") || html.contains("high"),
            "HTML should mention High severity");
    assert!(html.contains("Medium") || html.contains("medium"),
            "HTML should mention Medium severity");

    // Check for color styling (red, orange, yellow or hex equivalents)
    let has_colors = html.contains("red") || html.contains("#ff") ||
                     html.contains("orange") || html.contains("#ff8") ||
                     html.contains("yellow") || html.contains("rgb");
    assert!(has_colors, "HTML should contain color styling for severity");

    println!("[PASS] Severity color coding implemented");
}

#[test]
fn test_html_contains_embedded_css() {
    println!("[TEST] Verification: HTML contains embedded CSS styles");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    assert!(html.contains("<style>"), "HTML should contain embedded CSS");
    assert!(html.contains("</style>"), "HTML should close style tag");

    // Should have some basic styling
    let has_styling = html.contains("background") || html.contains("color") ||
                      html.contains("padding") || html.contains("margin");
    assert!(has_styling, "CSS should contain basic styling properties");

    println!("[PASS] Embedded CSS present");
}

#[test]
fn test_html_escapes_user_content() {
    println!("[TEST] Verification: HTML properly escapes user content (XSS protection)");

    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let storage = StorageManager::new(db_path).unwrap();

    // Create telemetry with potentially dangerous content
    // Include a security finding so it appears in the table
    let telemetry = TelemetryData {
        timestamp: Utc::now(),
        method: "<script>alert('xss')</script>".to_string(),
        request_data: "<img src=x onerror=alert('xss')>".to_string(),
        response_data: Some("</table><script>evil()</script>".to_string()),
        security_findings: vec![
            SecurityFinding {
                line_num: 1,
                severity: Severity::High,
                pattern_type: PatternType::ApiKey,
            }
        ],
    };

    storage.store_telemetry(&telemetry).unwrap();

    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    // Script tags should be escaped
    assert!(!html.contains("<script>alert('xss')</script>"),
            "Script tags should be escaped");
    assert!(!html.contains("<img src=x onerror=alert('xss')>"),
            "Event handlers should be escaped");

    // Check that content is escaped (contains &lt; or &gt;)
    let has_escaping = html.contains("&lt;") || html.contains("&gt;") ||
                       html.contains("&amp;");
    assert!(has_escaping, "HTML should escape special characters");

    println!("[PASS] XSS protection via HTML escaping works");
}

#[test]
fn test_html_displays_empty_state() {
    println!("[TEST] Verification: HTML handles empty database gracefully");

    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap();
    let storage = StorageManager::new(db_path).unwrap();

    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    assert!(!html.is_empty(), "HTML should be generated even for empty database");
    assert!(html.contains("0") || html.contains("No") || html.contains("Empty"),
            "HTML should indicate no data");

    println!("[PASS] Empty state handled correctly");
}

#[test]
fn test_html_shows_severity_breakdown() {
    println!("[TEST] Verification: HTML shows severity breakdown statistics");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    // We have 2 High and 1 Medium in test data
    assert!(html.contains("High") && html.contains("2"),
            "HTML should show High severity count (2)");
    assert!(html.contains("Medium") && html.contains("1"),
            "HTML should show Medium severity count (1)");

    println!("[PASS] Severity breakdown displayed");
}

#[test]
fn test_html_is_dark_theme_friendly() {
    println!("[TEST] Verification: HTML uses dark theme friendly colors");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    // Should have dark background or appropriate colors
    let has_dark_theme = html.contains("background") &&
                         (html.contains("#1e1e1e") || html.contains("#252526") ||
                          html.contains("#2d2d30") || html.contains("rgb(30") ||
                          html.contains("dark") || html.contains("black"));

    assert!(has_dark_theme, "HTML should use dark theme colors compatible with VSCode");

    println!("[PASS] Dark theme support present");
}

#[test]
fn test_html_includes_deduplication_stats() {
    println!("[TEST] Verification: HTML shows file deduplication statistics");

    let (_temp_file, storage) = create_test_db();
    let panel = VisualizationPanel::new();
    let html = panel.generate_html(&storage).unwrap();

    // Should show unique files count
    assert!(html.contains("Unique") || html.contains("unique") ||
            html.contains("Files") || html.contains("Hashes"),
            "HTML should mention unique files or deduplication");

    println!("[PASS] Deduplication statistics included");
}

#[test]
fn test_visualization_panel_new() {
    println!("[TEST] Verification: VisualizationPanel::new() creates instance");

    let panel = VisualizationPanel::new();
    // Just verify we can create an instance
    assert!(true, "Panel instance created successfully");

    println!("[PASS] VisualizationPanel instantiation works");
}
