use chrono::{DateTime, Utc};
use crate::security_scanner::{scan_content, SecurityFinding};

/// Represents a single telemetry data point capturing an LSP interaction
#[derive(Debug, Clone)]
pub struct TelemetryData {
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub request_data: String,
    pub response_data: Option<String>,
    pub security_findings: Vec<SecurityFinding>,
}

/// Collects telemetry data from LSP interactions
pub struct TelemetryCollector {
    data: Vec<TelemetryData>,
}

impl TelemetryCollector {
    /// Creates a new TelemetryCollector
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Captures an LSP method call with request and optional response
    /// Automatically scans the content for security vulnerabilities
    pub fn capture(&mut self, method: &str, request_data: &str, response_data: Option<&str>) {
        let timestamp = Utc::now();

        // Combine request and response data for security scanning
        let mut combined_content = request_data.to_string();
        if let Some(response) = response_data {
            combined_content.push('\n');
            combined_content.push_str(response);
        }

        // Scan for security vulnerabilities
        let security_findings = scan_content(&combined_content);

        let telemetry = TelemetryData {
            timestamp,
            method: method.to_string(),
            request_data: request_data.to_string(),
            response_data: response_data.map(|s| s.to_string()),
            security_findings,
        };

        self.data.push(telemetry);
    }

    /// Returns the number of captured interactions
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// Returns true if no data has been captured
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the most recent telemetry data
    pub fn get_latest(&self) -> Option<&TelemetryData> {
        self.data.last()
    }

    /// Returns all telemetry data
    pub fn get_all(&self) -> &[TelemetryData] {
        &self.data
    }

    /// Clears all telemetry data
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collector_is_empty() {
        let collector = TelemetryCollector::new();
        assert!(collector.is_empty());
        assert_eq!(collector.count(), 0);
    }

    #[test]
    fn test_capture_increments_count() {
        let mut collector = TelemetryCollector::new();
        collector.capture("test", "data", None);
        assert_eq!(collector.count(), 1);
        assert!(!collector.is_empty());
    }

    #[test]
    fn test_get_latest_returns_last_captured() {
        let mut collector = TelemetryCollector::new();
        collector.capture("method1", "data1", None);
        collector.capture("method2", "data2", None);

        let latest = collector.get_latest().unwrap();
        assert_eq!(latest.method, "method2");
    }

    #[test]
    fn test_clear_empties_collector() {
        let mut collector = TelemetryCollector::new();
        collector.capture("test", "data", None);
        assert!(!collector.is_empty());

        collector.clear();
        assert!(collector.is_empty());
    }
}
