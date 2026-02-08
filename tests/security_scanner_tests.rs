use vscode_agent_monitor::security_scanner::{scan_content, SecurityFinding, Severity, PatternType};

#[test]
fn test_detect_sk_api_key() {
    println!("TEST: Detecting sk- prefixed API keys");

    let content = r#"
        const API_KEY = "sk-1234567890abcdef";
        // Some other code
    "#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());
    assert!(!findings.is_empty(), "Should detect sk- prefixed API key");

    let sk_finding = findings.iter()
        .find(|f| matches!(f.pattern_type, PatternType::ApiKey));

    assert!(sk_finding.is_some(), "Should have ApiKey pattern type");
    let finding = sk_finding.unwrap();

    println!("FOUND: line={}, severity={:?}, pattern={:?}",
             finding.line_num, finding.severity, finding.pattern_type);

    assert_eq!(finding.line_num, 2, "Should be on line 2");
    assert!(matches!(finding.severity, Severity::High), "Should be High severity");
}

#[test]
fn test_detect_password_pattern() {
    println!("TEST: Detecting password patterns");

    let content = r#"
        password = "my_secret_password123"
        let user_password = "another_pass"
    "#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());
    assert!(!findings.is_empty(), "Should detect password patterns");

    let password_findings: Vec<_> = findings.iter()
        .filter(|f| matches!(f.pattern_type, PatternType::Password))
        .collect();

    println!("FOUND: {} password patterns", password_findings.len());
    assert!(password_findings.len() >= 1, "Should detect at least one password pattern");

    for finding in &password_findings {
        println!("PASSWORD: line={}, severity={:?}", finding.line_num, finding.severity);
        assert!(matches!(finding.severity, Severity::High), "Passwords should be High severity");
    }
}

#[test]
fn test_detect_api_key_pattern() {
    println!("TEST: Detecting api_key patterns");

    let content = r#"
        api_key = "abc123def456"
        const API_KEY = "xyz789"
        apiKey: "camelCase123"
    "#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());
    assert!(!findings.is_empty(), "Should detect api_key patterns");

    let api_key_findings: Vec<_> = findings.iter()
        .filter(|f| matches!(f.pattern_type, PatternType::ApiKey))
        .collect();

    println!("FOUND: {} api_key patterns", api_key_findings.len());
    assert!(api_key_findings.len() >= 1, "Should detect at least one api_key pattern");

    for finding in &api_key_findings {
        println!("API_KEY: line={}, severity={:?}", finding.line_num, finding.severity);
        assert!(matches!(finding.severity, Severity::High), "API keys should be High severity");
    }
}

#[test]
fn test_detect_high_entropy_strings() {
    println!("TEST: Detecting high-entropy strings");

    // High entropy string (Shannon entropy > 4.5)
    let content = r#"
        token = "9fK2mN8pQ7rT5sU1vW3xY6zA4bC0dE"
        secret = "aAaAaA"
    "#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());

    let high_entropy_findings: Vec<_> = findings.iter()
        .filter(|f| matches!(f.pattern_type, PatternType::HighEntropy))
        .collect();

    println!("FOUND: {} high-entropy patterns", high_entropy_findings.len());
    assert!(high_entropy_findings.len() >= 1, "Should detect at least one high-entropy string");

    for finding in &high_entropy_findings {
        println!("HIGH_ENTROPY: line={}, severity={:?}", finding.line_num, finding.severity);
        assert!(matches!(finding.severity, Severity::Medium), "High entropy should be Medium severity");
    }
}

#[test]
fn test_no_false_positives_on_normal_code() {
    println!("TEST: No false positives on normal code");

    let content = r#"
        fn hello_world() {
            println!("Hello, world!");
            let x = 42;
        }
    "#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());
    assert!(findings.is_empty(), "Should not detect secrets in normal code");
}

#[test]
fn test_multiple_findings_on_same_line() {
    println!("TEST: Multiple findings on same line");

    let content = r#"
        config = { "api_key": "sk-abc123def456", "password": "secret" }
    "#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());
    assert!(findings.len() >= 2, "Should detect multiple secrets on same line");

    for finding in &findings {
        println!("FOUND: line={}, pattern={:?}, severity={:?}",
                 finding.line_num, finding.pattern_type, finding.severity);
    }
}

#[test]
fn test_line_numbers_are_accurate() {
    println!("TEST: Line numbers are accurate");

    let content = r#"line 1
line 2
api_key = "secret123"
line 4
password = "pass123"
line 6"#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());
    assert!(!findings.is_empty(), "Should detect secrets");

    for finding in &findings {
        println!("FOUND: line={}, pattern={:?}", finding.line_num, finding.pattern_type);
        assert!(finding.line_num >= 1 && finding.line_num <= 6, "Line number should be between 1 and 6");
    }

    // Check specific line numbers
    let has_line_3 = findings.iter().any(|f| f.line_num == 3);
    let has_line_5 = findings.iter().any(|f| f.line_num == 5);

    assert!(has_line_3, "Should detect secret on line 3");
    assert!(has_line_5, "Should detect secret on line 5");
}

#[test]
fn test_severity_levels() {
    println!("TEST: Severity levels are correct");

    let content = r#"
        password = "pass123"
        token = "9fK2mN8pQ7rT5sU1vW3xY6zA4bC0dE"
    "#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());
    assert!(!findings.is_empty(), "Should detect secrets");

    for finding in &findings {
        println!("SEVERITY: line={}, pattern={:?}, severity={:?}",
                 finding.line_num, finding.pattern_type, finding.severity);

        match finding.pattern_type {
            PatternType::Password | PatternType::ApiKey => {
                assert!(matches!(finding.severity, Severity::High),
                       "Passwords and API keys should be High severity");
            }
            PatternType::HighEntropy => {
                assert!(matches!(finding.severity, Severity::Medium),
                       "High entropy should be Medium severity");
            }
        }
    }
}

#[test]
fn test_case_insensitive_detection() {
    println!("TEST: Case-insensitive pattern detection");

    let content = r#"
        PASSWORD = "pass123"
        Api_Key = "key123"
        APIKEY = "key456"
    "#;

    let findings = scan_content(content);

    println!("RESULT: Found {} findings", findings.len());
    assert!(findings.len() >= 2, "Should detect patterns regardless of case");

    for finding in &findings {
        println!("FOUND: line={}, pattern={:?}", finding.line_num, finding.pattern_type);
    }
}
