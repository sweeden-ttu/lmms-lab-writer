use crate::logger;
use crate::models::{CanvasPayload, VideoProvider};
use anyhow::{Context, Result};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use regex::Regex;

/// Extracted video link information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ExtractedVideo {
    pub url: String,
    pub title: Option<String>,
    pub provider: VideoProvider,
    pub course_id: Option<u64>,
    pub module_id: Option<u64>,
}

/// Video extraction summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoExtractionSummary {
    pub total_videos: usize,
    pub youtube_videos: usize,
    pub vimeo_videos: usize,
    pub mp4_videos: usize,
    pub other_videos: usize,
}

/// Extract video links from payload text content
fn extract_urls_from_text(text: &str) -> Vec<String> {
    let mut urls = Vec::new();
    
    // URL regex pattern - match http(s)://... until whitespace or end
    let url_pattern = Regex::new(r"(https?://[^\s\)]+)").unwrap();
    
    for cap in url_pattern.captures_iter(text) {
        if let Some(matched) = cap.get(1) {
            let url = matched.as_str().to_string();
            // Clean up trailing characters
            let url = url.trim_end_matches(|c| c == ')' || c == '\'' || c == '"');
            urls.push(url.to_string());
        }
    }
    
    urls
}

/// Check if a URL is a video URL
fn is_video_url(url: &str) -> bool {
    url.contains("youtube.com") 
        || url.contains("youtu.be")
        || url.contains("vimeo.com")
        || url.contains(".mp4")
        || url.contains("video")
        || url.contains("media")
}

/// Extract videos from a Canvas payload
///
/// # Arguments
/// * `payload` - The parsed Canvas payload
///
/// # Returns
/// * `Vec<ExtractedVideo>` - Vector of unique extracted videos
pub fn extract_videos(payload: &CanvasPayload) -> Vec<ExtractedVideo> {
    let mut videos = HashSet::new();
    
    for course in &payload.courses {
        for module in &course.modules {
            for item in &module.items {
                // Check item URL
                if let Some(url) = &item.url {
                    if is_video_url(url) {
                        let provider = VideoProvider::from_url(url);
                        let video = ExtractedVideo {
                            url: url.clone(),
                            title: Some(item.title.clone()),
                            provider,
                            course_id: Some(course.id),
                            module_id: Some(module.id),
                        };
                        videos.insert(video);
                        debug!(
                            "Extracted video from item {}: {}",
                            item.id, url
                        );
                    }
                }
            }
        }
    }
    
    let mut result: Vec<ExtractedVideo> = videos.into_iter().collect();
    result.sort_by(|a, b| a.url.cmp(&b.url));
    
    info!("Extracted {} unique videos from payload", result.len());
    result
}

/// Save extracted videos to a JSON file
///
/// # Arguments
/// * `videos` - Vector of extracted videos
/// * `output_path` - Path to save the videos file
///
/// # Returns
/// * `Result<()>` - Success or error
pub fn save_videos_to_file(videos: &[ExtractedVideo], output_path: &Path) -> Result<()> {
    debug!("Saving {} videos to {:?}", videos.len(), output_path);

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create parent directory: {:?}", parent)
        })?;
    }

    let json = serde_json::to_string_pretty(videos).with_context(|| {
        "Failed to serialize videos to JSON"
    })?;

    fs::write(output_path, json).with_context(|| {
        format!("Failed to write videos to file: {:?}", output_path)
    })?;

    info!("Successfully saved {} videos to {:?}", videos.len(), output_path);
    Ok(())
}

/// Get video statistics
pub fn get_video_stats(videos: &[ExtractedVideo]) -> VideoExtractionSummary {
    let mut youtube = 0;
    let mut vimeo = 0;
    let mut mp4 = 0;
    let mut other = 0;
    
    for video in videos {
        match video.provider {
            VideoProvider::YouTube => youtube += 1,
            VideoProvider::Vimeo => vimeo += 1,
            VideoProvider::Canvas => {
                if video.url.contains(".mp4") {
                    mp4 += 1;
                } else {
                    other += 1;
                }
            }
            VideoProvider::Other => {
                if video.url.contains(".mp4") {
                    mp4 += 1;
                } else {
                    other += 1;
                }
            }
        }
    }
    
    VideoExtractionSummary {
        total_videos: videos.len(),
        youtube_videos: youtube,
        vimeo_videos: vimeo,
        mp4_videos: mp4,
        other_videos: other,
    }
}

/// Extract videos and save to file in one operation
///
/// # Arguments
/// * `payload` - The parsed Canvas payload
/// * `output_path` - Path to save the videos file
///
/// # Returns
/// * `Result<VideoExtractionSummary>` - Summary of extraction
pub fn extract_and_save_videos(
    payload: &CanvasPayload,
    output_path: &Path,
) -> Result<VideoExtractionSummary> {
    logger::log_operation_start("extract_videos");

    let videos = extract_videos(payload);
    let stats = get_video_stats(&videos);

    if !videos.is_empty() {
        save_videos_to_file(&videos, output_path)?;
    }

    let mut details = HashMap::new();
    details.insert(
        "total_videos".to_string(),
        stats.total_videos.to_string(),
    );
    details.insert(
        "youtube_videos".to_string(),
        stats.youtube_videos.to_string(),
    );
    details.insert(
        "vimeo_videos".to_string(),
        stats.vimeo_videos.to_string(),
    );
    details.insert(
        "mp4_videos".to_string(),
        stats.mp4_videos.to_string(),
    );

    logger::log_operation_complete("extract_videos", details);
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Course, Module, ModuleItem};
    use tempfile::TempDir;

    fn create_test_payload_with_videos() -> CanvasPayload {
        CanvasPayload {
            courses: vec![Course {
                id: 1,
                name: "Video Course".to_string(),
                code: Some("V101".to_string()),
                modules: vec![
                    Module {
                        id: 1,
                        name: "Module 1".to_string(),
                        items: vec![
                            ModuleItem {
                                id: 1,
                                title: "YouTube Video".to_string(),
                                item_type: "video".to_string(),
                                url: Some("https://www.youtube.com/watch?v=abc123".to_string()),
                                content_id: None,
                                indent: None,
                                completion_requirement: None,
                            },
                            ModuleItem {
                                id: 2,
                                title: "Vimeo Video".to_string(),
                                item_type: "video".to_string(),
                                url: Some("https://vimeo.com/123456".to_string()),
                                content_id: None,
                                indent: None,
                                completion_requirement: None,
                            },
                            ModuleItem {
                                id: 3,
                                title: "MP4 File".to_string(),
                                item_type: "file".to_string(),
                                url: Some("https://example.com/video.mp4".to_string()),
                                content_id: None,
                                indent: None,
                                completion_requirement: None,
                            },
                        ],
                        position: None,
                    },
                ],
                enrollments: vec![],
            }],
            user: None,
            metadata: None,
        }
    }

    #[test]
    fn test_extract_videos() {
        let payload = create_test_payload_with_videos();
        let videos = extract_videos(&payload);

        assert_eq!(videos.len(), 3);
        assert!(videos.iter().any(|v| v.url.contains("youtube")));
        assert!(videos.iter().any(|v| v.url.contains("vimeo")));
        assert!(videos.iter().any(|v| v.url.contains("mp4")));
    }

    #[test]
    fn test_extract_videos_empty_payload() {
        let payload = CanvasPayload {
            courses: vec![],
            user: None,
            metadata: None,
        };

        let videos = extract_videos(&payload);
        assert_eq!(videos.len(), 0);
    }

    #[test]
    fn test_get_video_stats() {
        let payload = create_test_payload_with_videos();
        let videos = extract_videos(&payload);
        let stats = get_video_stats(&videos);

        assert_eq!(stats.total_videos, 3);
        assert_eq!(stats.youtube_videos, 1);
        assert_eq!(stats.vimeo_videos, 1);
        assert_eq!(stats.mp4_videos, 1);
    }

    #[test]
    fn test_save_videos_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("videos.json");

        let videos = vec![
            ExtractedVideo {
                url: "https://www.youtube.com/watch?v=abc123".to_string(),
                title: Some("Test Video".to_string()),
                provider: VideoProvider::YouTube,
                course_id: Some(1),
                module_id: Some(1),
            },
        ];

        let result = save_videos_to_file(&videos, &output_path);
        assert!(result.is_ok());
        assert!(output_path.exists());

        let content = fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("youtube"));
    }

    #[test]
    fn test_extract_and_save_videos() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("videos.json");
        let payload = create_test_payload_with_videos();

        let result = extract_and_save_videos(&payload, &output_path);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_videos, 3);
        assert!(output_path.exists());
    }

    #[test]
    fn test_extract_and_save_videos_empty() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("videos.json");
        let payload = CanvasPayload {
            courses: vec![],
            user: None,
            metadata: None,
        };

        let result = extract_and_save_videos(&payload, &output_path);
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.total_videos, 0);
        assert!(!output_path.exists());
    }

    #[test]
    fn test_is_video_url() {
        assert!(is_video_url("https://www.youtube.com/watch?v=abc"));
        assert!(is_video_url("https://youtu.be/abc"));
        assert!(is_video_url("https://vimeo.com/123"));
        assert!(is_video_url("https://example.com/video.mp4"));
        assert!(is_video_url("https://example.com/media/video"));
        assert!(!is_video_url("https://example.com/document.pdf"));
    }

    #[test]
    fn test_extract_urls_from_text() {
        let text = "Check this video https://www.youtube.com/watch?v=abc123 and this one https://vimeo.com/456";
        let urls = extract_urls_from_text(text);
        
        assert!(urls.len() >= 2);
        assert!(urls.iter().any(|u| u.contains("youtube")));
        assert!(urls.iter().any(|u| u.contains("vimeo")));
    }
}
