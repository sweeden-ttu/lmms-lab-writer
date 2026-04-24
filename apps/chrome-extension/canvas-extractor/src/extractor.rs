use crate::logger;
use crate::models::{CanvasPayload, Course, Enrollment, Grades};
use anyhow::{Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Grade information extracted from a course
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedGrade {
    pub course_id: u64,
    pub course_name: String,
    pub user_id: u64,
    pub current_score: Option<f64>,
    pub final_score: Option<f64>,
    pub current_grade: Option<String>,
    pub final_grade: Option<String>,
    pub role: String,
}

/// Grade extraction summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeExtractionSummary {
    pub total_courses: usize,
    pub total_enrollments: usize,
    pub grades_extracted: usize,
}

/// Extract grades from a Canvas payload
///
/// # Arguments
/// * `payload` - The parsed Canvas payload
///
/// # Returns
/// * `Vec<ExtractedGrade>` - Vector of extracted grades
pub fn extract_grades(payload: &CanvasPayload) -> Vec<ExtractedGrade> {
    let mut extracted_grades = Vec::new();

    for course in &payload.courses {
        for enrollment in &course.enrollments {
            if let Some(grades) = &enrollment.grades {
                let grade = ExtractedGrade {
                    course_id: course.id,
                    course_name: course.name.clone(),
                    user_id: enrollment.user_id,
                    current_score: grades.current_score,
                    final_score: grades.final_score,
                    current_grade: grades.current_grade.clone(),
                    final_grade: grades.final_grade.clone(),
                    role: enrollment.role.clone(),
                };
                extracted_grades.push(grade);
                debug!(
                    "Extracted grade for user {} in course {}: {:?}",
                    enrollment.user_id, course.id, grades.current_score
                );
            }
        }
    }

    info!("Extracted {} grades from payload", extracted_grades.len());
    extracted_grades
}

/// Save extracted grades to a JSON file
///
/// # Arguments
/// * `grades` - Vector of extracted grades
/// * `output_path` - Path to save the grades file
///
/// # Returns
/// * `Result<()>` - Success or error
pub fn save_grades_to_file(grades: &[ExtractedGrade], output_path: &Path) -> Result<()> {
    debug!("Saving {} grades to {:?}", grades.len(), output_path);

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent directory: {:?}", parent)
        })?;
    }

    let json = serde_json::to_string_pretty(grades).with_context(|| {
        "Failed to serialize grades to JSON"
    })?;

    fs::write(output_path, json).with_context(|| {
        format!("Failed to write grades to file: {:?}", output_path)
    })?;

    info!("Successfully saved {} grades to {:?}", grades.len(), output_path);
    Ok(())
}

/// Get grade statistics from extracted grades
pub fn get_grade_stats(grades: &[ExtractedGrade]) -> GradeExtractionSummary {
    // Count unique courses
    let unique_courses: std::collections::HashSet<u64> =
        grades.iter().map(|g| g.course_id).collect();

    // Count unique users
    let unique_users: std::collections::HashSet<u64> =
        grades.iter().map(|g| g.user_id).collect();

    GradeExtractionSummary {
        total_courses: unique_courses.len(),
        total_enrollments: unique_users.len(),
        grades_extracted: grades.len(),
    }
}

/// Extract grades and save to file in one operation
///
/// # Arguments
/// * `payload` - The parsed Canvas payload
/// * `output_path` - Path to save the grades file
///
/// # Returns
/// * `Result<GradeExtractionSummary>` - Summary of extraction
pub fn extract_and_save_grades(
    payload: &CanvasPayload,
    output_path: &Path,
) -> Result<GradeExtractionSummary> {
    logger::log_operation_start("extract_grades");

    let grades = extract_grades(payload);
    let stats = get_grade_stats(&grades);

    if !grades.is_empty() {
        save_grades_to_file(&grades, output_path)?;
    }

    let mut details = HashMap::new();
    details.insert(
        "grades_extracted".to_string(),
        stats.grades_extracted.to_string(),
    );
    details.insert(
        "total_courses".to_string(),
        stats.total_courses.to_string(),
    );
    details.insert(
        "total_enrollments".to_string(),
        stats.total_enrollments.to_string(),
    );

    logger::log_operation_complete("extract_grades", details);
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_payload() -> CanvasPayload {
        CanvasPayload {
            courses: vec![
                Course {
                    id: 1,
                    name: "Math 101".to_string(),
                    code: Some("M101".to_string()),
                    modules: vec![],
                    enrollments: vec![
                        Enrollment {
                            id: 1,
                            user_id: 100,
                            role: "student".to_string(),
                            grades: Some(Grades {
                                current_score: Some(85.5),
                                final_score: Some(82.0),
                                current_grade: Some("B+".to_string()),
                                final_grade: Some("B".to_string()),
                            }),
                        },
                        Enrollment {
                            id: 2,
                            user_id: 101,
                            role: "student".to_string(),
                            grades: Some(Grades {
                                current_score: Some(95.0),
                                final_score: Some(93.5),
                                current_grade: Some("A".to_string()),
                                final_grade: Some("A".to_string()),
                            }),
                        },
                    ],
                },
                Course {
                    id: 2,
                    name: "English 101".to_string(),
                    code: Some("E101".to_string()),
                    modules: vec![],
                    enrollments: vec![Enrollment {
                        id: 3,
                        user_id: 100,
                        role: "student".to_string(),
                        grades: Some(Grades {
                            current_score: Some(78.5),
                            final_score: None,
                            current_grade: Some("C+".to_string()),
                            final_grade: None,
                        }),
                    }],
                },
            ],
            user: None,
            metadata: None,
        }
    }

    #[test]
    fn test_extract_grades() {
        let payload = create_test_payload();
        let grades = extract_grades(&payload);

        assert_eq!(grades.len(), 3);
        assert_eq!(grades[0].course_id, 1);
        assert_eq!(grades[0].user_id, 100);
        assert_eq!(grades[0].current_score, Some(85.5));
        assert_eq!(grades[0].final_score, Some(82.0));
    }

    #[test]
    fn test_extract_grades_empty_payload() {
        let payload = CanvasPayload {
            courses: vec![],
            user: None,
            metadata: None,
        };

        let grades = extract_grades(&payload);
        assert_eq!(grades.len(), 0);
    }

    #[test]
    fn test_extract_grades_no_grades() {
        let payload = CanvasPayload {
            courses: vec![Course {
                id: 1,
                name: "Test Course".to_string(),
                code: None,
                modules: vec![],
                enrollments: vec![Enrollment {
                    id: 1,
                    user_id: 100,
                    role: "student".to_string(),
                    grades: None, // No grades
                }],
            }],
            user: None,
            metadata: None,
        };

        let grades = extract_grades(&payload);
        assert_eq!(grades.len(), 0);
    }

    #[test]
    fn test_save_grades_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("grades.json");

        let grades = vec![
            ExtractedGrade {
                course_id: 1,
                course_name: "Math 101".to_string(),
                user_id: 100,
                current_score: Some(85.5),
                final_score: Some(82.0),
                current_grade: Some("B+".to_string()),
                final_grade: Some("B".to_string()),
                role: "student".to_string(),
            },
        ];

        let result = save_grades_to_file(&grades, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify file content
        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Math 101"));
        assert!(content.contains("85.5"));
    }

    #[test]
    fn test_get_grade_stats() {
        let payload = create_test_payload();
        let grades = extract_grades(&payload);
        let stats = get_grade_stats(&grades);

        assert_eq!(stats.grades_extracted, 3);
        assert_eq!(stats.total_courses, 2);
        assert_eq!(stats.total_enrollments, 2); // users 100 and 101
    }

    #[test]
    fn test_extract_and_save_grades() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("grades.json");
        let payload = create_test_payload();

        let result = extract_and_save_grades(&payload, &output_path);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.grades_extracted, 3);
        assert!(output_path.exists());
    }

    #[test]
    fn test_extract_and_save_grades_empty() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("grades.json");
        let payload = CanvasPayload {
            courses: vec![],
            user: None,
            metadata: None,
        };

        let result = extract_and_save_grades(&payload, &output_path);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.grades_extracted, 0);
        // File should not be created if no grades
        assert!(!output_path.exists());
    }
}
