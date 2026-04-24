use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Canvas course
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: u64,
    pub name: String,
    pub code: Option<String>,
    #[serde(default)]
    pub modules: Vec<Module>,
    #[serde(default)]
    pub enrollments: Vec<Enrollment>,
}

/// Represents a module within a course
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub items: Vec<ModuleItem>,
    #[serde(default)]
    pub position: Option<u32>,
}

/// Represents a module item (file, video, assignment, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleItem {
    pub id: u64,
    pub title: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub url: Option<String>,
    pub content_id: Option<u64>,
    pub indent: Option<u32>,
    pub completion_requirement: Option<CompletionRequirement>,
}

/// Completion requirement for a module item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequirement {
    #[serde(rename = "type")]
    pub requirement_type: String,
    pub min_score: Option<f64>,
    pub completed: Option<bool>,
}

/// Represents a file (course material)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: u64,
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub content_type: String,
    #[serde(default)]
    pub folder_id: Option<u64>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Represents a student enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    pub id: u64,
    pub user_id: u64,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub grades: Option<Grades>,
}

/// Represents grades for an enrollment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grades {
    pub current_score: Option<f64>,
    pub final_score: Option<f64>,
    pub current_grade: Option<String>,
    pub final_grade: Option<String>,
}

/// Represents a video link found in content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VideoLink {
    pub url: String,
    pub title: Option<String>,
    pub provider: VideoProvider,
}

/// Represents the source/provider of a video
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum VideoProvider {
    YouTube,
    Vimeo,
    Canvas,
    Other,
}

impl VideoProvider {
    pub fn from_url(url: &str) -> Self {
        if url.contains("youtube.com") || url.contains("youtu.be") {
            VideoProvider::YouTube
        } else if url.contains("vimeo.com") {
            VideoProvider::Vimeo
        } else if url.contains("canvas") {
            VideoProvider::Canvas
        } else {
            VideoProvider::Other
        }
    }
}

/// Represents the parsed Canvas payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasPayload {
    #[serde(default)]
    pub courses: Vec<Course>,
    #[serde(default)]
    pub user: Option<User>,
    #[serde(default)]
    pub metadata: Option<PayloadMetadata>,
}

/// Represents user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: Option<String>,
    pub login_id: Option<String>,
}

/// Represents metadata about the payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadMetadata {
    pub exported_at: Option<String>,
    pub version: Option<String>,
    pub institution: Option<String>,
}

/// Summary statistics for extracted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionSummary {
    pub total_courses: usize,
    pub total_modules: usize,
    pub total_files: usize,
    pub total_videos: usize,
    pub total_enrollments: usize,
    pub grades_extracted: usize,
    pub download_stats: DownloadStats,
}

/// Statistics about downloads
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadStats {
    pub total_downloads: usize,
    pub successful_downloads: usize,
    pub failed_downloads: usize,
    pub total_bytes_downloaded: u64,
}

/// Log entry for tracking operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub operation: String,
    pub details: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_provider_youtube() {
        assert_eq!(
            VideoProvider::from_url("https://www.youtube.com/watch?v=abc123"),
            VideoProvider::YouTube
        );
    }

    #[test]
    fn test_video_provider_youtu_be() {
        assert_eq!(
            VideoProvider::from_url("https://youtu.be/abc123"),
            VideoProvider::YouTube
        );
    }

    #[test]
    fn test_video_provider_vimeo() {
        assert_eq!(
            VideoProvider::from_url("https://vimeo.com/123456"),
            VideoProvider::Vimeo
        );
    }

    #[test]
    fn test_video_provider_canvas() {
        assert_eq!(
            VideoProvider::from_url("https://canvas.edu/media/abc"),
            VideoProvider::Canvas
        );
    }

    #[test]
    fn test_video_provider_other() {
        assert_eq!(
            VideoProvider::from_url("https://example.com/video"),
            VideoProvider::Other
        );
    }

    #[test]
    fn test_course_serialization() {
        let course = Course {
            id: 1,
            name: "Test Course".to_string(),
            code: Some("CS101".to_string()),
            modules: vec![],
            enrollments: vec![],
        };

        let json = serde_json::to_string(&course).unwrap();
        let deserialized: Course = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.name, "Test Course");
    }
}
