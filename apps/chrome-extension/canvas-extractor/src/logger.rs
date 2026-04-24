use chrono::Local;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    /// Thread-safe operation log
    static ref OPERATION_LOG: Mutex<Vec<OperationLog>> = Mutex::new(Vec::new());
}

/// Represents a single logged operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub timestamp: String,
    pub operation: String,
    pub status: OperationStatus,
    pub details: HashMap<String, String>,
}

/// Status of an operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationStatus {
    Started,
    InProgress,
    Completed,
    Failed,
}

impl OperationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            OperationStatus::Started => "STARTED",
            OperationStatus::InProgress => "IN_PROGRESS",
            OperationStatus::Completed => "COMPLETED",
            OperationStatus::Failed => "FAILED",
        }
    }
}

/// Log application startup
pub fn log_startup() {
    let timestamp = Local::now().to_rfc3339();
    info!("=== Canvas Payload Parser Startup ===");
    info!("Timestamp: {}", timestamp);
    log_operation("startup", OperationStatus::Started, HashMap::new());
}

/// Log an operation with timestamp and details
pub fn log_operation(
    operation: &str,
    status: OperationStatus,
    mut details: HashMap<String, String>,
) {
    let timestamp = Local::now().to_rfc3339();
    details.insert("status".to_string(), status.as_str().to_string());
    
    info!(
        "{} - Operation: {} ({})",
        timestamp,
        operation,
        status.as_str()
    );

    let log_entry = OperationLog {
        timestamp: timestamp.clone(),
        operation: operation.to_string(),
        status,
        details,
    };

    let mut log = OPERATION_LOG.lock().unwrap();
    log.push(log_entry);
}

/// Log a major operation with start message
pub fn log_operation_start(operation: &str) {
    info!("Starting operation: {}", operation);
    log_operation(operation, OperationStatus::Started, HashMap::new());
}

/// Log operation completion
pub fn log_operation_complete(operation: &str, details: HashMap<String, String>) {
    info!("Completed operation: {}", operation);
    log_operation(operation, OperationStatus::Completed, details);
}

/// Log operation failure
pub fn log_operation_failed(operation: &str, error: &str) {
    let mut details = HashMap::new();
    details.insert("error".to_string(), error.to_string());
    log_operation(operation, OperationStatus::Failed, details);
}

/// Log a summary with statistics
pub fn log_summary(stats: &HashMap<String, usize>) {
    info!("=== Summary ===");
    for (key, value) in stats {
        info!("{}: {}", key, value);
    }
    log_operation("summary", OperationStatus::Completed, 
        stats.iter().map(|(k, v)| (k.clone(), v.to_string())).collect());
}

/// Get all operation logs
pub fn get_operation_logs() -> Vec<OperationLog> {
    let log = OPERATION_LOG.lock().unwrap();
    log.clone()
}

/// Clear operation logs
pub fn clear_operation_logs() {
    let mut log = OPERATION_LOG.lock().unwrap();
    log.clear();
}

/// Export operation logs as JSON
pub fn export_logs_json() -> String {
    let log = OPERATION_LOG.lock().unwrap();
    serde_json::to_string_pretty(&*log).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to ensure test isolation
    fn setup_test() {
        clear_operation_logs();
    }

    #[test]
    fn test_operation_status_as_str() {
        assert_eq!(OperationStatus::Started.as_str(), "STARTED");
        assert_eq!(OperationStatus::InProgress.as_str(), "IN_PROGRESS");
        assert_eq!(OperationStatus::Completed.as_str(), "COMPLETED");
        assert_eq!(OperationStatus::Failed.as_str(), "FAILED");
    }

    #[test]
    fn test_log_operation() {
        setup_test();
        let mut details = HashMap::new();
        details.insert("test".to_string(), "value".to_string());
        
        log_operation("test_op", OperationStatus::Completed, details);
        
        let logs = get_operation_logs();
        // Filter to just our test operation
        let test_logs: Vec<_> = logs.iter().filter(|l| l.operation == "test_op").collect();
        assert_eq!(test_logs.len(), 1);
        assert_eq!(test_logs[0].status, OperationStatus::Completed);
    }

    #[test]
    fn test_log_operation_start() {
        setup_test();
        log_operation_start("test_operation");
        
        let logs = get_operation_logs();
        let test_logs: Vec<_> = logs.iter().filter(|l| l.operation == "test_operation").collect();
        assert_eq!(test_logs.len(), 1);
        assert_eq!(test_logs[0].status, OperationStatus::Started);
    }

    #[test]
    fn test_log_operation_complete() {
        setup_test();
        let mut details = HashMap::new();
        details.insert("result".to_string(), "success".to_string());
        
        log_operation_complete("test_op_complete", details);
        
        let logs = get_operation_logs();
        let test_logs: Vec<_> = logs.iter().filter(|l| l.operation == "test_op_complete").collect();
        assert_eq!(test_logs.len(), 1);
        assert_eq!(test_logs[0].status, OperationStatus::Completed);
    }

    #[test]
    fn test_log_operation_failed() {
        setup_test();
        log_operation_failed("test_op_failed", "Error occurred");
        
        let logs = get_operation_logs();
        let test_logs: Vec<_> = logs.iter().filter(|l| l.operation == "test_op_failed").collect();
        assert_eq!(test_logs.len(), 1);
        assert_eq!(test_logs[0].status, OperationStatus::Failed);
        assert_eq!(test_logs[0].details.get("error").unwrap(), "Error occurred");
    }

    #[test]
    fn test_export_logs_json() {
        setup_test();
        log_operation_start("test_json");
        
        let json = export_logs_json();
        assert!(json.contains("test_json"));
        assert!(json.contains("STARTED"));
    }

    #[test]
    fn test_log_summary() {
        setup_test();
        let mut stats = HashMap::new();
        stats.insert("courses".to_string(), 5);
        stats.insert("modules".to_string(), 20);
        
        log_summary(&stats);
        
        let logs = get_operation_logs();
        let test_logs: Vec<_> = logs.iter().filter(|l| l.operation == "summary").collect();
        assert_eq!(test_logs.len(), 1);
    }
}
