use crate::storage_manager::StorageManager;
use rusqlite::Result as SqlResult;

/// Generates HTML WebView UI for displaying telemetry statistics and security findings
pub struct VisualizationPanel;

impl VisualizationPanel {
    /// Creates a new VisualizationPanel instance
    pub fn new() -> Self {
        Self
    }

    /// Generates complete HTML page with statistics and security findings
    pub fn generate_html(&self, storage: &StorageManager) -> SqlResult<String> {
        // Gather statistics
        let total_events = storage.count_telemetry_logs()?;
        let unique_files = storage.count_file_hashes()?;
        let (total_findings, high_count, medium_count, low_count) = self.get_severity_counts(storage)?;

        // Get all security findings with context
        let findings = self.get_all_findings_with_context(storage)?;

        // Generate HTML
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VSCode Agent Monitor - Dashboard</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            background-color: #1e1e1e;
            color: #cccccc;
            padding: 20px;
            line-height: 1.6;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}

        h1 {{
            color: #ffffff;
            margin-bottom: 30px;
            font-size: 28px;
            border-bottom: 2px solid #007acc;
            padding-bottom: 10px;
        }}

        h2 {{
            color: #ffffff;
            margin-top: 30px;
            margin-bottom: 15px;
            font-size: 22px;
        }}

        .dashboard {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}

        .stat-card {{
            background-color: #252526;
            border: 1px solid #3c3c3c;
            border-radius: 8px;
            padding: 20px;
            transition: transform 0.2s, box-shadow 0.2s;
        }}

        .stat-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(0, 122, 204, 0.2);
        }}

        .stat-card h3 {{
            color: #007acc;
            font-size: 14px;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 10px;
        }}

        .stat-value {{
            font-size: 36px;
            font-weight: bold;
            color: #ffffff;
        }}

        .stat-label {{
            font-size: 12px;
            color: #888888;
            margin-top: 5px;
        }}

        .severity-breakdown {{
            display: flex;
            gap: 15px;
            flex-wrap: wrap;
            margin-top: 10px;
        }}

        .severity-item {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}

        .severity-badge {{
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 12px;
            font-weight: bold;
        }}

        .severity-high {{
            background-color: #ff4444;
            color: #ffffff;
        }}

        .severity-medium {{
            background-color: #ff8c00;
            color: #ffffff;
        }}

        .severity-low {{
            background-color: #ffd700;
            color: #000000;
        }}

        .findings-table-wrapper {{
            background-color: #252526;
            border: 1px solid #3c3c3c;
            border-radius: 8px;
            overflow: hidden;
        }}

        table {{
            width: 100%;
            border-collapse: collapse;
        }}

        thead {{
            background-color: #2d2d30;
        }}

        th {{
            padding: 12px;
            text-align: left;
            font-weight: 600;
            color: #ffffff;
            border-bottom: 2px solid #007acc;
        }}

        td {{
            padding: 12px;
            border-bottom: 1px solid #3c3c3c;
        }}

        tbody tr:hover {{
            background-color: #2d2d30;
        }}

        .method-cell {{
            font-family: 'Courier New', monospace;
            color: #9cdcfe;
            font-size: 13px;
        }}

        .line-num {{
            color: #888888;
            font-size: 12px;
        }}

        .pattern-type {{
            font-weight: 500;
            color: #ce9178;
        }}

        .empty-state {{
            text-align: center;
            padding: 60px 20px;
            color: #888888;
        }}

        .empty-state h3 {{
            font-size: 20px;
            margin-bottom: 10px;
            color: #cccccc;
        }}

        .timestamp {{
            color: #888888;
            font-size: 11px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>VSCode Agent Monitor Dashboard</h1>

        <div class="dashboard">
            <div class="stat-card">
                <h3>Total Events</h3>
                <div class="stat-value">{}</div>
                <div class="stat-label">Telemetry interactions captured</div>
            </div>

            <div class="stat-card">
                <h3>Unique Files</h3>
                <div class="stat-value">{}</div>
                <div class="stat-label">Deduplicated content blocks</div>
            </div>

            <div class="stat-card">
                <h3>Security Findings</h3>
                <div class="stat-value">{}</div>
                <div class="stat-label">Total vulnerabilities detected</div>
                <div class="severity-breakdown">
                    <div class="severity-item">
                        <span class="severity-badge severity-high">High</span>
                        <span>{}</span>
                    </div>
                    <div class="severity-item">
                        <span class="severity-badge severity-medium">Medium</span>
                        <span>{}</span>
                    </div>
                    <div class="severity-item">
                        <span class="severity-badge severity-low">Low</span>
                        <span>{}</span>
                    </div>
                </div>
            </div>
        </div>

        <h2>Security Findings</h2>
        {}

    </div>
</body>
</html>"#,
            total_events,
            unique_files,
            total_findings,
            high_count,
            medium_count,
            low_count,
            self.generate_findings_table(&findings)
        );

        Ok(html)
    }

    /// Gets severity counts from all security findings
    fn get_severity_counts(&self, storage: &StorageManager) -> SqlResult<(usize, usize, usize, usize)> {
        let conn = storage.get_connection();

        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM security_findings",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let high: i64 = conn.query_row(
            "SELECT COUNT(*) FROM security_findings WHERE severity = 'High'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let medium: i64 = conn.query_row(
            "SELECT COUNT(*) FROM security_findings WHERE severity = 'Medium'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let low: i64 = conn.query_row(
            "SELECT COUNT(*) FROM security_findings WHERE severity = 'Low'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok((total as usize, high as usize, medium as usize, low as usize))
    }

    /// Gets all security findings with their associated telemetry context
    fn get_all_findings_with_context(&self, storage: &StorageManager) -> SqlResult<Vec<FindingRow>> {
        let conn = storage.get_connection();

        let mut stmt = conn.prepare(
            "SELECT
                sf.severity,
                sf.pattern_type,
                sf.line_num,
                t.method,
                t.timestamp
             FROM security_findings sf
             INNER JOIN telemetry_logs t ON sf.telemetry_id = t.id
             ORDER BY t.timestamp DESC, sf.line_num"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(FindingRow {
                severity: row.get(0)?,
                pattern_type: row.get(1)?,
                line_num: row.get::<_, i64>(2)? as usize,
                method: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    /// Generates HTML table for security findings
    fn generate_findings_table(&self, findings: &[FindingRow]) -> String {
        if findings.is_empty() {
            return r#"<div class="empty-state">
                <h3>No Security Findings</h3>
                <p>No vulnerabilities detected in captured telemetry data.</p>
            </div>"#.to_string();
        }

        let mut table = String::from(r#"<div class="findings-table-wrapper">
            <table>
                <thead>
                    <tr>
                        <th>Severity</th>
                        <th>Pattern Type</th>
                        <th>Line</th>
                        <th>Method</th>
                        <th>Timestamp</th>
                    </tr>
                </thead>
                <tbody>"#);

        for finding in findings {
            let severity_class = match finding.severity.as_str() {
                "High" => "severity-high",
                "Medium" => "severity-medium",
                "Low" => "severity-low",
                _ => "severity-low",
            };

            // HTML escape the content
            let escaped_method = html_escape(&finding.method);
            let escaped_pattern = html_escape(&finding.pattern_type);

            table.push_str(&format!(
                r#"
                    <tr>
                        <td><span class="severity-badge {}">{}</span></td>
                        <td class="pattern-type">{}</td>
                        <td class="line-num">{}</td>
                        <td class="method-cell">{}</td>
                        <td class="timestamp">{}</td>
                    </tr>"#,
                severity_class,
                html_escape(&finding.severity),
                escaped_pattern,
                finding.line_num,
                escaped_method,
                format_timestamp(finding.timestamp)
            ));
        }

        table.push_str(r#"
                </tbody>
            </table>
        </div>"#);

        table
    }
}

impl Default for VisualizationPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper struct for finding row data
struct FindingRow {
    severity: String,
    pattern_type: String,
    line_num: usize,
    method: String,
    timestamp: i64,
}

/// HTML escapes a string to prevent XSS attacks
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Formats Unix timestamp into human-readable string
fn format_timestamp(timestamp: i64) -> String {
    use chrono::DateTime;

    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Invalid timestamp".to_string()
    }
}

// Extension methods for StorageManager to support visualization queries
// These are implemented in storage_manager.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("normal text"), "normal text");
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a&b"), "a&amp;b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
        assert_eq!(html_escape("'single'"), "&#x27;single&#x27;");
    }

    #[test]
    fn test_html_escape_complex() {
        let input = "<img src=x onerror=alert('xss')>";
        let output = html_escape(input);
        assert!(!output.contains("<img"));
        assert!(output.contains("&lt;"));
        assert!(output.contains("&gt;"));
    }
}
