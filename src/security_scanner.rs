use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternType {
    ApiKey,
    Password,
    HighEntropy,
}

#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub line_num: usize,
    pub severity: Severity,
    pub pattern_type: PatternType,
}

/// Scans content for security vulnerabilities
pub fn scan_content(content: &str) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    // Compile regex patterns
    let sk_pattern = Regex::new(r"sk-[a-zA-Z0-9]{16,}").unwrap();
    let password_pattern = Regex::new(r#"(?i)(["']?password["']?|["']?passwd["']?|["']?pwd["']?)\s*[=:]\s*"#).unwrap();
    let api_key_pattern = Regex::new(r#"(?i)(["']?api[_-]?key["']?|["']?apikey["']?)\s*[=:]"#).unwrap();

    // Process each line
    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;

        // Check for sk- prefixed API keys
        if sk_pattern.is_match(line) {
            findings.push(SecurityFinding {
                line_num,
                severity: Severity::High,
                pattern_type: PatternType::ApiKey,
            });
        }

        // Check for password patterns
        if password_pattern.is_match(line) {
            findings.push(SecurityFinding {
                line_num,
                severity: Severity::High,
                pattern_type: PatternType::Password,
            });
        }

        // Check for api_key patterns (excluding sk- which was already caught)
        if api_key_pattern.is_match(line) {
            // Only add if not already detected as sk- pattern
            let already_has_apikey = findings.iter()
                .any(|f| f.line_num == line_num && matches!(f.pattern_type, PatternType::ApiKey));

            if !already_has_apikey {
                findings.push(SecurityFinding {
                    line_num,
                    severity: Severity::High,
                    pattern_type: PatternType::ApiKey,
                });
            }
        }

        // Check for high-entropy strings
        if let Some(high_entropy_finding) = check_high_entropy(line, line_num) {
            findings.push(high_entropy_finding);
        }
    }

    findings
}

/// Checks for high-entropy strings in a line
fn check_high_entropy(line: &str, line_num: usize) -> Option<SecurityFinding> {
    // Extract potential secrets (strings in quotes)
    let string_pattern = Regex::new(r#"["']([a-zA-Z0-9]{20,})["']"#).unwrap();

    for caps in string_pattern.captures_iter(line) {
        if let Some(matched) = caps.get(1) {
            let string_value = matched.as_str();
            let entropy = calculate_shannon_entropy(string_value);

            if entropy > 4.5 {
                return Some(SecurityFinding {
                    line_num,
                    severity: Severity::Medium,
                    pattern_type: PatternType::HighEntropy,
                });
            }
        }
    }

    None
}

/// Calculates Shannon entropy of a string
fn calculate_shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }

    let mut char_counts: HashMap<char, usize> = HashMap::new();
    for c in s.chars() {
        *char_counts.entry(c).or_insert(0) += 1;
    }

    let len = s.len() as f64;
    let mut entropy = 0.0;

    for count in char_counts.values() {
        let probability = *count as f64 / len;
        entropy -= probability * probability.log2();
    }

    entropy
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_entropy() {
        // Low entropy (repeated characters)
        let low_entropy = calculate_shannon_entropy("aaaaaa");
        assert!(low_entropy < 1.0, "Repeated characters should have low entropy");

        // High entropy (random characters)
        let high_entropy = calculate_shannon_entropy("9fK2mN8pQ7rT5sU1vW3xY6zA4bC0dE");
        assert!(high_entropy > 4.5, "Random string should have high entropy");

        // Medium entropy
        let medium_entropy = calculate_shannon_entropy("password123");
        assert!(medium_entropy > 2.0 && medium_entropy < 4.0);
    }

    #[test]
    fn test_empty_string() {
        let entropy = calculate_shannon_entropy("");
        assert_eq!(entropy, 0.0);
    }
}
