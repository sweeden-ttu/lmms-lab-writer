use crate::models::CanvasPayload;
use anyhow::{Context, Result};
use log::{debug, error, info};
use std::fs;
use std::path::Path;

/// Parse Canvas payload from a JSON file
///
/// # Arguments
/// * `path` - Path to the JSON file (typically /tmp/course_payload.json)
///
/// # Returns
/// * `Result<CanvasPayload>` - Parsed payload or error
pub fn parse_payload<P: AsRef<Path>>(path: P) -> Result<CanvasPayload> {
    let path = path.as_ref();
    debug!("Attempting to parse payload from: {:?}", path);

    // Check if file exists
    if !path.exists() {
        error!(
            "Payload file does not exist: {}",
            path.display()
        );
        return Err(anyhow::anyhow!(
            "Payload file not found: {}",
            path.display()
        ));
    }

    // Read the file
    let contents = fs::read_to_string(path).with_context(|| {
        format!("Failed to read payload file: {}", path.display())
    })?;

    debug!(
        "Read {} bytes from payload file",
        contents.len()
    );

    // Parse JSON
    let payload: CanvasPayload = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse JSON from file: {}", path.display()))?;

    info!("Successfully parsed Canvas payload");
    info!(
        "Found {} courses",
        payload.courses.len()
    );

    // Log module count
    let total_modules: usize = payload.courses.iter().map(|c| c.modules.len()).sum();
    info!("Found {} total modules", total_modules);

    Ok(payload)
}

/// Validate the structure of a parsed payload
///
/// # Arguments
/// * `payload` - The parsed CanvasPayload to validate
///
/// # Returns
/// * `Result<()>` - Success or error details
pub fn validate_payload(payload: &CanvasPayload) -> Result<()> {
    debug!("Validating payload structure");

    if payload.courses.is_empty() {
        debug!("Warning: No courses found in payload");
    }

    // Check each course has valid structure
    for (idx, course) in payload.courses.iter().enumerate() {
        if course.id == 0 {
            error!("Course at index {} has invalid ID 0", idx);
            return Err(anyhow::anyhow!(
                "Course at index {} has invalid ID",
                idx
            ));
        }

        if course.name.is_empty() {
            debug!(
                "Warning: Course {} has empty name",
                course.id
            );
        }

        debug!(
            "Course {}: {} with {} modules",
            course.id,
            course.name,
            course.modules.len()
        );
    }

    info!("Payload validation passed");
    Ok(())
}

/// Get summary statistics from parsed payload
pub fn get_payload_stats(payload: &CanvasPayload) -> PayloadStats {
    let total_courses = payload.courses.len();
    let total_modules: usize = payload.courses.iter().map(|c| c.modules.len()).sum();
    let total_items: usize = payload
        .courses
        .iter()
        .flat_map(|c| &c.modules)
        .map(|m| m.items.len())
        .sum();
    let total_enrollments: usize = payload.courses.iter().map(|c| c.enrollments.len()).sum();

    PayloadStats {
        total_courses,
        total_modules,
        total_items,
        total_enrollments,
    }
}

/// Statistics about the parsed payload
#[derive(Debug, Clone)]
pub struct PayloadStats {
    pub total_courses: usize,
    pub total_modules: usize,
    pub total_items: usize,
    pub total_enrollments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Course, Enrollment, Module};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_payload() {
        // Create a temporary file with valid JSON
        let json_content = r#"{
            "courses": [
                {
                    "id": 1,
                    "name": "Test Course",
                    "code": "CS101",
                    "modules": [],
                    "enrollments": []
                }
            ],
            "user": null,
            "metadata": null
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let payload = parse_payload(temp_file.path()).unwrap();
        assert_eq!(payload.courses.len(), 1);
        assert_eq!(payload.courses[0].id, 1);
        assert_eq!(payload.courses[0].name, "Test Course");
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_payload("/tmp/nonexistent_canvas_payload_12345.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"{ invalid json }").unwrap();
        temp_file.flush().unwrap();

        let result = parse_payload(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_payload_empty() {
        let payload = CanvasPayload {
            courses: vec![],
            user: None,
            metadata: None,
        };
        let result = validate_payload(&payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_payload_valid() {
        let payload = CanvasPayload {
            courses: vec![Course {
                id: 1,
                name: "Test".to_string(),
                code: None,
                modules: vec![],
                enrollments: vec![],
            }],
            user: None,
            metadata: None,
        };
        let result = validate_payload(&payload);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_payload_invalid_course_id() {
        let payload = CanvasPayload {
            courses: vec![Course {
                id: 0, // Invalid ID
                name: "Test".to_string(),
                code: None,
                modules: vec![],
                enrollments: vec![],
            }],
            user: None,
            metadata: None,
        };
        let result = validate_payload(&payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_payload_stats() {
        let payload = CanvasPayload {
            courses: vec![
                Course {
                    id: 1,
                    name: "Course 1".to_string(),
                    code: None,
                    modules: vec![
                        Module {
                            id: 1,
                            name: "Module 1".to_string(),
                            items: vec![],
                            position: None,
                        },
                        Module {
                            id: 2,
                            name: "Module 2".to_string(),
                            items: vec![],
                            position: None,
                        },
                    ],
                    enrollments: vec![Enrollment {
                        id: 1,
                        user_id: 1,
                        role: "student".to_string(),
                        grades: None,
                    }],
                },
                Course {
                    id: 2,
                    name: "Course 2".to_string(),
                    code: None,
                    modules: vec![],
                    enrollments: vec![],
                },
            ],
            user: None,
            metadata: None,
        };

        let stats = get_payload_stats(&payload);
        assert_eq!(stats.total_courses, 2);
        assert_eq!(stats.total_modules, 2);
        assert_eq!(stats.total_enrollments, 1);
    }
}
