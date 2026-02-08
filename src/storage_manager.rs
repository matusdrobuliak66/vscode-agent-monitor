use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqlResult, params, OptionalExtension};
use crate::security_scanner::{SecurityFinding, Severity, PatternType};
use crate::telemetry_collector::TelemetryData;

/// Manages SQLite database for storing telemetry data
pub struct StorageManager {
    conn: Connection,
}

impl StorageManager {
    /// Creates a new StorageManager and initializes the database
    pub fn new(db_path: &str) -> SqlResult<Self> {
        let conn = Connection::open(db_path)?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        let mut storage = Self { conn };
        storage.init_db()?;
        Ok(storage)
    }

    /// Initializes database schema with all required tables
    fn init_db(&mut self) -> SqlResult<()> {
        // Create file_hashes table for content deduplication
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS file_hashes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                hash TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL
            )",
            [],
        )?;

        // Create telemetry_logs table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS telemetry_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                method TEXT NOT NULL,
                request_data_hash INTEGER NOT NULL,
                response_data_hash INTEGER,
                FOREIGN KEY (request_data_hash) REFERENCES file_hashes(id),
                FOREIGN KEY (response_data_hash) REFERENCES file_hashes(id)
            )",
            [],
        )?;

        // Create index on timestamp for faster queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_telemetry_timestamp
             ON telemetry_logs(timestamp)",
            [],
        )?;

        // Create index on method for faster queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_telemetry_method
             ON telemetry_logs(method)",
            [],
        )?;

        // Create security_findings table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS security_findings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                telemetry_id INTEGER NOT NULL,
                line_num INTEGER NOT NULL,
                severity TEXT NOT NULL,
                pattern_type TEXT NOT NULL,
                FOREIGN KEY (telemetry_id) REFERENCES telemetry_logs(id)
            )",
            [],
        )?;

        // Create index on telemetry_id for faster joins
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_findings_telemetry
             ON security_findings(telemetry_id)",
            [],
        )?;

        Ok(())
    }

    /// Computes BLAKE3 hash of content
    fn compute_hash(content: &str) -> String {
        let hash = blake3::hash(content.as_bytes());
        hash.to_hex().to_string()
    }

    /// Gets or creates a file hash entry, returning its ID
    fn get_or_create_hash(&self, content: &str) -> SqlResult<i64> {
        let hash = Self::compute_hash(content);

        // Try to find existing hash
        let existing: Option<i64> = self.conn.query_row(
            "SELECT id FROM file_hashes WHERE hash = ?1",
            params![hash],
            |row| row.get(0),
        ).optional()?;

        if let Some(id) = existing {
            Ok(id)
        } else {
            // Insert new hash
            self.conn.execute(
                "INSERT INTO file_hashes (hash, content) VALUES (?1, ?2)",
                params![hash, content],
            )?;
            Ok(self.conn.last_insert_rowid())
        }
    }

    /// Stores telemetry data and returns the telemetry ID
    pub fn store_telemetry(&self, telemetry: &TelemetryData) -> SqlResult<i64> {
        // Get or create hash for request data
        let request_hash_id = self.get_or_create_hash(&telemetry.request_data)?;

        // Get or create hash for response data (if present)
        let response_hash_id = if let Some(ref response) = telemetry.response_data {
            Some(self.get_or_create_hash(response)?)
        } else {
            None
        };

        // Insert telemetry log
        let timestamp = telemetry.timestamp.timestamp();
        self.conn.execute(
            "INSERT INTO telemetry_logs (timestamp, method, request_data_hash, response_data_hash)
             VALUES (?1, ?2, ?3, ?4)",
            params![timestamp, telemetry.method, request_hash_id, response_hash_id],
        )?;

        let telemetry_id = self.conn.last_insert_rowid();

        // Store security findings
        for finding in &telemetry.security_findings {
            let severity_str = match finding.severity {
                Severity::High => "High",
                Severity::Medium => "Medium",
                Severity::Low => "Low",
            };

            let pattern_type_str = match finding.pattern_type {
                PatternType::ApiKey => "ApiKey",
                PatternType::Password => "Password",
                PatternType::HighEntropy => "HighEntropy",
            };

            self.conn.execute(
                "INSERT INTO security_findings (telemetry_id, line_num, severity, pattern_type)
                 VALUES (?1, ?2, ?3, ?4)",
                params![telemetry_id, finding.line_num as i64, severity_str, pattern_type_str],
            )?;
        }

        Ok(telemetry_id)
    }

    /// Retrieves security findings for a specific telemetry record
    pub fn get_security_findings(&self, telemetry_id: i64) -> SqlResult<Vec<SecurityFinding>> {
        let mut stmt = self.conn.prepare(
            "SELECT line_num, severity, pattern_type
             FROM security_findings
             WHERE telemetry_id = ?1
             ORDER BY line_num"
        )?;

        let findings = stmt.query_map(params![telemetry_id], |row| {
            let line_num: i64 = row.get(0)?;
            let severity_str: String = row.get(1)?;
            let pattern_type_str: String = row.get(2)?;

            let severity = match severity_str.as_str() {
                "High" => Severity::High,
                "Medium" => Severity::Medium,
                "Low" => Severity::Low,
                _ => Severity::Low,
            };

            let pattern_type = match pattern_type_str.as_str() {
                "ApiKey" => PatternType::ApiKey,
                "Password" => PatternType::Password,
                "HighEntropy" => PatternType::HighEntropy,
                _ => PatternType::HighEntropy,
            };

            Ok(SecurityFinding {
                line_num: line_num as usize,
                severity,
                pattern_type,
            })
        })?;

        findings.collect()
    }

    /// Queries telemetry data by timestamp range
    pub fn query_by_time(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> SqlResult<Vec<TelemetryData>> {
        let start_ts = start.timestamp();
        let end_ts = end.timestamp();

        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.timestamp, t.method, h1.content, h2.content
             FROM telemetry_logs t
             INNER JOIN file_hashes h1 ON t.request_data_hash = h1.id
             LEFT JOIN file_hashes h2 ON t.response_data_hash = h2.id
             WHERE t.timestamp >= ?1 AND t.timestamp <= ?2
             ORDER BY t.timestamp"
        )?;

        let rows = stmt.query_map(params![start_ts, end_ts], |row| {
            let telemetry_id: i64 = row.get(0)?;
            let timestamp: i64 = row.get(1)?;
            let method: String = row.get(2)?;
            let request_data: String = row.get(3)?;
            let response_data: Option<String> = row.get(4)?;

            Ok((telemetry_id, timestamp, method, request_data, response_data))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (telemetry_id, timestamp, method, request_data, response_data) = row?;
            let security_findings = self.get_security_findings(telemetry_id)?;

            results.push(TelemetryData {
                timestamp: DateTime::from_timestamp(timestamp, 0).unwrap(),
                method,
                request_data,
                response_data,
                security_findings,
            });
        }

        Ok(results)
    }

    /// Queries telemetry data by LSP method
    pub fn query_by_method(&self, method: &str) -> SqlResult<Vec<TelemetryData>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.id, t.timestamp, t.method, h1.content, h2.content
             FROM telemetry_logs t
             INNER JOIN file_hashes h1 ON t.request_data_hash = h1.id
             LEFT JOIN file_hashes h2 ON t.response_data_hash = h2.id
             WHERE t.method = ?1
             ORDER BY t.timestamp"
        )?;

        let rows = stmt.query_map(params![method], |row| {
            let telemetry_id: i64 = row.get(0)?;
            let timestamp: i64 = row.get(1)?;
            let method: String = row.get(2)?;
            let request_data: String = row.get(3)?;
            let response_data: Option<String> = row.get(4)?;

            Ok((telemetry_id, timestamp, method, request_data, response_data))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (telemetry_id, timestamp, method, request_data, response_data) = row?;
            let security_findings = self.get_security_findings(telemetry_id)?;

            results.push(TelemetryData {
                timestamp: DateTime::from_timestamp(timestamp, 0).unwrap(),
                method,
                request_data,
                response_data,
                security_findings,
            });
        }

        Ok(results)
    }

    /// Counts total telemetry logs in database
    pub fn count_telemetry_logs(&self) -> SqlResult<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM telemetry_logs",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Counts unique file hashes in database
    pub fn count_file_hashes(&self) -> SqlResult<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM file_hashes",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Counts total security findings in database
    pub fn count_security_findings(&self) -> SqlResult<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM security_findings",
            [],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// Gets a reference to the underlying database connection
    /// Used by visualization_panel for complex queries
    pub(crate) fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash_consistency() {
        let content = "test content";
        let hash1 = StorageManager::compute_hash(content);
        let hash2 = StorageManager::compute_hash(content);
        assert_eq!(hash1, hash2, "Same content should produce same hash");
    }

    #[test]
    fn test_compute_hash_different_content() {
        let hash1 = StorageManager::compute_hash("content1");
        let hash2 = StorageManager::compute_hash("content2");
        assert_ne!(hash1, hash2, "Different content should produce different hashes");
    }
}
