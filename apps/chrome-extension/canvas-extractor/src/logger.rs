use chrono::Local;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

lazy_static::lazy_static! {
    /// Thread-safe operation log
    static ref OPERATION_LOG: Mutex<Vec<OperationLog>> = Mutex::new(Vec::new());
    /// Track operation start times for duration calculation
    static ref OPERATION_TIMERS: Mutex<HashMap<String, u64>> = Mutex::new(HashMap::new());
}

/// Represents a single logged operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub timestamp: String,
    pub operation: String,
    pub status: OperationStatus,
    pub duration_ms: Option<u64>,
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

/// Get current time in milliseconds since Unix epoch
fn get_current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Log application startup
pub fn log_startup() {
    let timestamp = Local::now().to_rfc3339();
    info!("=== Canvas Payload Parser Startup ===");
    info!("Timestamp: {}", timestamp);
    log_operation_start("startup");
}

/// Log an operation with timestamp and details
fn log_operation(
    operation: &str,
    status: OperationStatus,
    mut details: HashMap<String, String>,
    duration_ms: Option<u64>,
) {
    let timestamp = Local::now().to_rfc3339();
    details.insert("status".to_string(), status.as_str().to_string());
    
    let duration_str = duration_ms
        .map(|d| format!(" ({}ms)", d))
        .unwrap_or_default();
    
    info!(
        "{} - Operation: {} {}{}", 
        timestamp,
        operation,
        status.as_str(),
        duration_str
    );

    let log_entry = OperationLog {
        timestamp: timestamp.clone(),
        operation: operation.to_string(),
        status,
        duration_ms,
        details,
    };

    let mut log = OPERATION_LOG.lock().unwrap();
    log.push(log_entry);
}

/// Log a major operation with start message and timer
pub fn log_operation_start(operation: &str) {
    let start_time = get_current_time_ms();
    info!("Starting operation: {}", operation);
    
    // Store start time for duration tracking
    let mut timers = OPERATION_TIMERS.lock().unwrap();
    timers.insert(operation.to_string(), start_time);
    
    log_operation(operation, OperationStatus::Started, HashMap::new(), None);
}

/// Log operation completion with duration
pub fn log_operation_complete(operation: &str, details: HashMap<String, String>) {
    let end_time = get_current_time_ms();
    
    // Calculate duration
    let duration_ms = {
        let timers = OPERATION_TIMERS.lock().unwrap();
        timers.get(operation)
            .map(|start_time| end_time - start_time)
    };
    
    info!("Completed operation: {} ({:?}ms)", operation, duration_ms);
    log_operation(operation, OperationStatus::Completed, details, duration_ms);
    
    // Clean up timer
    let mut timers = OPERATION_TIMERS.lock().unwrap();
    timers.remove(operation);
}

/// Log operation failure with duration and error context
pub fn log_operation_failed(operation: &str, error: &str) {
    let end_time = get_current_time_ms();
    
    // Calculate duration
    let duration_ms = {
        let timers = OPERATION_TIMERS.lock().unwrap();
        timers.get(operation)
            .map(|start_time| end_time - start_time)
    };
    
    let mut details = HashMap::new();
    details.insert("error".to_string(), error.to_string());
    
    error!("Operation failed: {} - {} ({:?}ms)", operation, error, duration_ms);
    log_operation(operation, OperationStatus::Failed, details, duration_ms);
    
    // Clean up timer
    let mut timers = OPERATION_TIMERS.lock().unwrap();
    timers.remove(operation);
}

/// Log a summary with statistics
pub fn log_summary(stats: &HashMap<String, usize>) {
    info!("=== Summary ===");
    for (key, value) in stats {
        info!("{}: {}", key, value);
    }
    log_operation("summary", OperationStatus::Completed, 
        stats.iter().map(|(k, v)| (k.clone(), v.to_string())).collect(),
        None);
}

/// Log an error with context
pub fn log_error(context: &str, error: &str, details: Option<HashMap<String, String>>) {
    let mut error_details = details.unwrap_or_default();
    error_details.insert("context".to_string(), context.to_string());
    
    error!("Error in {}: {}", context, error);
    log_operation(&format!("error_{}", context), OperationStatus::Failed, error_details, None);
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

/// Clear operation timers
pub fn clear_operation_timers() {
    let mut timers = OPERATION_TIMERS.lock().unwrap();
    timers.clear();
}

/// Export operation logs as JSON
pub fn export_logs_json() -> String {
    let log = OPERATION_LOG.lock().unwrap();
    serde_json::to_string_pretty(&*log).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    lazy_static::lazy_static! {
        static ref TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);
    }

    // Helper to ensure test isolation with unique operation names
    fn setup_test() -> String {
        clear_operation_logs();
        clear_operation_timers();
        let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        format!("test_{}", id)
    }

    #[test]
    fn test_operation_status_as_str() {
        assert_eq!(OperationStatus::Started.as_str(), "STARTED");
        assert_eq!(OperationStatus::InProgress.as_str(), "IN_PROGRESS");
        assert_eq!(OperationStatus::Completed.as_str(), "COMPLETED");
        assert_eq!(OperationStatus::Failed.as_str(), "FAILED");
    }

    #[test]
    fn test_log_operation_with_duration() {
        let _test_id = setup_test();
        let mut details = HashMap::new();
        details.insert("test".to_string(), "value".to_string());
        
        log_operation("test_op_dur", OperationStatus::Completed, details, Some(100));
        
        let logs = get_operation_logs();
        let test_logs: Vec<_> = logs.iter().filter(|l| l.operation == "test_op_dur").collect();
        assert_eq!(test_logs.len(), 1);
        assert_eq!(test_logs[0].duration_ms, Some(100));
    }

    #[test]
    fn test_log_operation_start_and_complete() {
        let test_id = setup_test();
        let op_name = format!("flow_{}", test_id);
        
        log_operation_start(&op_name);
        
        // Verify started log exists
        let logs = get_operation_logs();
        let started_logs: Vec<_> = logs.iter()
            .filter(|l| l.operation == op_name && l.status == OperationStatus::Started)
            .collect();
        assert_eq!(started_logs.len(), 1, "Should have exactly one started log");
        
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        log_operation_complete(&op_name, HashMap::new());
        
        // Verify completed log exists with duration
        let logs = get_operation_logs();
        let completed_logs: Vec<_> = logs.iter()
            .filter(|l| l.operation == op_name && l.status == OperationStatus::Completed)
            .collect();
        assert_eq!(completed_logs.len(), 1, "Should have exactly one completed log");
        assert!(completed_logs[0].duration_ms.is_some(), "Duration should be captured");
        assert!(completed_logs[0].duration_ms.unwrap() >= 9, "Duration should be at least 9ms");
    }

    #[test]
    fn test_log_operation_failed_with_duration() {
        let test_id = setup_test();
        let op_name = format!("fail_{}", test_id);
        
        log_operation_start(&op_name);
        
        std::thread::sleep(std::time::Duration::from_millis(5));
        
        log_operation_failed(&op_name, "Test error");
        
        let logs = get_operation_logs();
        let failed_logs: Vec<_> = logs.iter()
            .filter(|l| l.operation == op_name && l.status == OperationStatus::Failed)
            .collect();
        assert_eq!(failed_logs.len(), 1, "Should have exactly one failed log");
        assert!(failed_logs[0].duration_ms.is_some(), "Duration should be captured on failed operation");
        assert!(failed_logs[0].duration_ms.unwrap() >= 4, "Duration should be at least 4ms");
        assert_eq!(failed_logs[0].details.get("error").unwrap(), "Test error");
    }

    #[test]
    fn test_log_error_with_context() {
        let test_id = setup_test();
        let mut details = HashMap::new();
        details.insert("code".to_string(), "500".to_string());
        
        log_error(&test_id, "Invalid JSON format", Some(details));
        
        let logs = get_operation_logs();
        let error_logs: Vec<_> = logs.iter()
            .filter(|l| l.operation == format!("error_{}", test_id))
            .collect();
        assert_eq!(error_logs.len(), 1);
        assert_eq!(error_logs[0].details.get("context").unwrap(), &test_id);
    }

    #[test]
    fn test_export_logs_json_with_duration() {
        let test_id = setup_test();
        let op_name = format!("dur_json_{}", test_id);
        
        log_operation_start(&op_name);
        log_operation_complete(&op_name, HashMap::new());
        
        let json = export_logs_json();
        assert!(json.contains(&op_name));
        assert!(json.contains("duration_ms"));
    }

    #[test]
    fn test_log_summary() {
        let _test_id = setup_test();
        let mut stats = HashMap::new();
        stats.insert("courses".to_string(), 5);
        stats.insert("modules".to_string(), 20);
        
        log_summary(&stats);
        
        let logs = get_operation_logs();
        let summary_logs: Vec<_> = logs.iter()
            .filter(|l| l.operation == "summary")
            .collect();
        assert_eq!(summary_logs.len(), 1);
    }

    #[test]
    fn test_get_current_time_ms() {
        let time1 = get_current_time_ms();
        std::thread::sleep(std::time::Duration::from_millis(5));
        let time2 = get_current_time_ms();
        
        assert!(time2 >= time1);
        assert!(time2 - time1 >= 4); // Allow some timing variance
    }
}
