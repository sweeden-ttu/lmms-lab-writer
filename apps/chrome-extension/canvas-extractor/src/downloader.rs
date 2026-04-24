use crate::logger;
use anyhow::{Context, Result};
use log::{debug, error, info};
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Download configuration
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    pub base_dir: PathBuf,
    pub output_dir: PathBuf,
    pub timeout_seconds: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        DownloadConfig {
            base_dir: PathBuf::from("./downloads"),
            output_dir: PathBuf::from("./output"),
            timeout_seconds: 30,
        }
    }
}

/// Download statistics
#[derive(Debug, Clone, Default)]
pub struct DownloadStats {
    pub total_downloads: usize,
    pub successful_downloads: usize,
    pub failed_downloads: usize,
    pub total_bytes_downloaded: u64,
}

/// Initialize all required directories
///
/// # Arguments
/// * `config` - Download configuration
///
/// # Returns
/// * `Result<()>` - Success or error
pub fn initialize_directories(config: &DownloadConfig) -> Result<()> {
    info!("Initializing directories");
    
    // Create downloads directory
    fs::create_dir_all(&config.base_dir).with_context(|| {
        format!("Failed to create downloads directory: {:?}", config.base_dir)
    })?;
    info!("Created downloads directory: {:?}", config.base_dir);
    
    // Create output directory
    fs::create_dir_all(&config.output_dir).with_context(|| {
        format!("Failed to create output directory: {:?}", config.output_dir)
    })?;
    info!("Created output directory: {:?}", config.output_dir);
    
    logger::log_operation_complete("initialize_directories", std::collections::HashMap::new());
    Ok(())
}

/// Download a single file
///
/// # Arguments
/// * `url` - The URL to download
/// * `course_id` - The course ID
/// * `module_id` - The module ID
/// * `filename` - The filename to save as
/// * `config` - Download configuration
///
/// # Returns
/// * `Result<u64>` - Number of bytes downloaded
pub async fn download_file(
    url: &str,
    course_id: u64,
    module_id: u64,
    filename: &str,
    config: &DownloadConfig,
) -> Result<u64> {
    debug!("Downloading file from URL: {}", url);

    // Create directory structure
    let dir = create_directory_structure(course_id, module_id, config)?;

    // Prepare file path
    let file_path = dir.join(filename);
    debug!("Download destination: {:?}", file_path);

    // Skip if file already exists
    if file_path.exists() {
        info!("File already exists, skipping: {:?}", file_path);
        let metadata = fs::metadata(&file_path)
            .with_context(|| format!("Failed to read metadata for {:?}", file_path))?;
        return Ok(metadata.len());
    }

    // Download the file with timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.timeout_seconds))
        .build()?;

    let response = client.get(url).send().await.with_context(|| {
        format!("Failed to download file from: {}", url)
    })?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP error downloading {}: {}",
            url,
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read response bytes from: {}", url))?;

    let content_len = bytes.len() as u64;
    debug!("Downloaded {} bytes", content_len);

    // Write file asynchronously
    let mut file = File::create(&file_path)
        .await
        .with_context(|| format!("Failed to create file: {:?}", file_path))?;

    file.write_all(&bytes)
        .await
        .with_context(|| format!("Failed to write file: {:?}", file_path))?;

    info!(
        "Successfully downloaded {} ({} bytes)",
        filename, content_len
    );

    Ok(content_len)
}

/// Download multiple files from a module
///
/// # Arguments
/// * `files` - Vector of tuples (url, filename)
/// * `course_id` - The course ID
/// * `module_id` - The module ID
/// * `config` - Download configuration
///
/// # Returns
/// * `DownloadStats` - Statistics about the download operation
pub async fn download_module_files(
    files: Vec<(String, String)>,
    course_id: u64,
    module_id: u64,
    config: &DownloadConfig,
) -> DownloadStats {
    let mut stats = DownloadStats {
        total_downloads: files.len(),
        successful_downloads: 0,
        failed_downloads: 0,
        total_bytes_downloaded: 0,
    };

    for (url, filename) in files {
        match download_file(&url, course_id, module_id, &filename, config).await {
            Ok(bytes) => {
                stats.successful_downloads += 1;
                stats.total_bytes_downloaded += bytes;
                debug!(
                    "Download successful: {} ({})",
                    filename, stats.successful_downloads
                );
            }
            Err(e) => {
                stats.failed_downloads += 1;
                error!("Failed to download {}: {}", filename, e);
                logger::log_operation_failed(&format!("download_{}", filename), &e.to_string());
            }
        }
    }

    info!(
        "Module download complete: {} successful, {} failed",
        stats.successful_downloads, stats.failed_downloads
    );

    stats
}

/// Create directory structure for course/module organization
///
/// # Arguments
/// * `course_id` - The course ID
/// * `module_id` - The module ID
/// * `config` - Download configuration
///
/// # Returns
/// * `Result<PathBuf>` - The path to the created directory
pub fn create_directory_structure(
    course_id: u64,
    module_id: u64,
    config: &DownloadConfig,
) -> Result<PathBuf> {
    let dir = config
        .base_dir
        .join(format!("course_{}", course_id))
        .join(format!("module_{}", module_id));

    fs::create_dir_all(&dir).with_context(|| {
        format!("Failed to create directory: {:?}", dir)
    })?;

    debug!("Created directory: {:?}", dir);
    Ok(dir)
}

/// Cleanup old downloaded files (optional maintenance)
pub fn cleanup_downloads(base_dir: &Path, days_old: u64) -> Result<usize> {
    let mut removed_count = 0;

    if !base_dir.exists() {
        return Ok(0);
    }

    for entry in fs::read_dir(base_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            removed_count += cleanup_downloads(&path, days_old)?;
        } else {
            // Check file age
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        let seconds_per_day = 86400;
                        if elapsed.as_secs() > days_old * seconds_per_day {
                            fs::remove_file(&path)?;
                            removed_count += 1;
                            debug!("Removed old file: {:?}", path);
                        }
                    }
                }
            }
        }
    }

    Ok(removed_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_initialize_directories() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig {
            base_dir: temp_dir.path().join("downloads"),
            output_dir: temp_dir.path().join("output"),
            timeout_seconds: 30,
        };

        let result = initialize_directories(&config);
        assert!(result.is_ok());
        assert!(config.base_dir.exists());
        assert!(config.output_dir.exists());
    }

    #[test]
    fn test_initialize_directories_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig {
            base_dir: temp_dir.path().join("downloads"),
            output_dir: temp_dir.path().join("output"),
            timeout_seconds: 30,
        };

        // Create directories twice - should not error
        assert!(initialize_directories(&config).is_ok());
        assert!(initialize_directories(&config).is_ok());
    }

    #[test]
    fn test_create_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config = DownloadConfig {
            base_dir: temp_dir.path().to_path_buf(),
            output_dir: temp_dir.path().join("output"),
            timeout_seconds: 30,
        };

        let result = create_directory_structure(1, 2, &config);
        assert!(result.is_ok());

        let dir = result.unwrap();
        assert!(dir.exists());
        assert!(dir.to_string_lossy().contains("course_1"));
        assert!(dir.to_string_lossy().contains("module_2"));
    }

    #[test]
    fn test_download_config_default() {
        let config = DownloadConfig::default();
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.base_dir.to_string_lossy(), "./downloads");
        assert_eq!(config.output_dir.to_string_lossy(), "./output");
    }

    #[test]
    fn test_download_stats_default() {
        let stats = DownloadStats::default();
        assert_eq!(stats.total_downloads, 0);
        assert_eq!(stats.successful_downloads, 0);
        assert_eq!(stats.failed_downloads, 0);
        assert_eq!(stats.total_bytes_downloaded, 0);
    }

    #[tokio::test]
    async fn test_download_module_files_empty() {
        let config = DownloadConfig::default();
        let stats = download_module_files(vec![], 1, 1, &config).await;
        assert_eq!(stats.total_downloads, 0);
        assert_eq!(stats.successful_downloads, 0);
    }

    #[test]
    fn test_cleanup_downloads_nonexistent_dir() {
        let result = cleanup_downloads(Path::new("/nonexistent/path"), 7);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_cleanup_downloads_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = cleanup_downloads(temp_dir.path(), 7);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
