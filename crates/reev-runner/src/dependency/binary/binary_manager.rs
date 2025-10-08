//! Binary manager implementation for downloading and caching dependencies

//! This module handles downloading surfpool binaries with proper platform detection
//! and URL construction for the txtx/surfpool repository.

use super::{BinaryAcquisitionResult, BinaryInfo, Platform, Version};
use anyhow::Result;
use flate2::read::GzDecoder;
use reqwest::Client;
use std::path::PathBuf;
use std::time::Duration;
use std::{io::Read, path::Path};
use tar::Archive;
use tokio::fs;
use tracing::{info, warn};

/// Binary manager for handling external dependencies
pub struct BinaryManager {
    client: Client,
    cache_dir: String,
    #[allow(dead_code)]
    install_dir: String,
    prefer_binary: bool,
}

impl BinaryManager {
    /// Create a new binary manager
    pub fn new(cache_dir: String, install_dir: String, prefer_binary: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("reev-runner/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            cache_dir,
            install_dir,
            prefer_binary,
        }
    }

    /// Get or build surfpool binary
    pub async fn get_or_build_surfpool(&self) -> Result<BinaryAcquisitionResult> {
        info!("Getting surfpool binary...");

        let platform = Platform::current();
        let arch = crate::dependency::binary::Architecture::current();

        // Try cache first
        if self.prefer_binary {
            if let Ok(cached) = self.get_cached_surfpool().await {
                return Ok(BinaryAcquisitionResult::Cached(cached));
            }
        }

        // Try download from GitHub
        if let Ok(downloaded) = self.download_surfpool(&platform, &arch).await {
            return Ok(BinaryAcquisitionResult::Downloaded(downloaded));
        }

        // Throw error
        panic!("Expect surfpool binary");
    }

    /// Get cached surfpool binary
    async fn get_cached_surfpool(&self) -> Result<PathBuf> {
        info!("Checking for cached surfpool binary...");

        let cache_path = PathBuf::from(&self.cache_dir).join("surfpool");

        if cache_path.exists() {
            // Check if binary is executable
            if cfg!(unix) {
                use std::os::unix::fs::PermissionsExt;
                let metadata = fs::metadata(&cache_path).await?;
                let perms = metadata.permissions();
                if perms.mode() & 0o111 == 0 {
                    // Make executable
                    let mut new_perms = perms.clone();
                    new_perms.set_mode(perms.mode() | 0o755);
                    fs::set_permissions(&cache_path, new_perms).await?;
                }
            }

            info!("Found cached surfpool binary at: {}", cache_path.display());
            Ok(cache_path)
        } else {
            Err(anyhow::anyhow!("No cached surfpool binary found"))
        }
    }

    /// Download surfpool from GitHub releases
    async fn download_surfpool(
        &self,
        platform: &Platform,
        arch: &crate::dependency::binary::Architecture,
    ) -> Result<PathBuf> {
        info!("Downloading surfpool binary for {}-{}...", platform, arch);

        // Get actual OS platform for filename construction
        let actual_platform = self.detect_os_platform();
        let actual_arch = self.detect_os_architecture();

        // Construct download URL based on detected platform and architecture
        let filename = match (&actual_platform, &actual_arch) {
            (
                crate::dependency::binary::Platform::Linux,
                crate::dependency::binary::Architecture::X86_64,
            ) => "surfpool-linux-x86_64.tar.gz",
            (
                crate::dependency::binary::Platform::MacOS,
                crate::dependency::binary::Architecture::X86_64,
            ) => "surfpool-darwin-x86_64.tar.gz",
            (
                crate::dependency::binary::Platform::MacOS,
                crate::dependency::binary::Architecture::Aarch64,
            ) => "surfpool-darwin-arm64.tar.gz",
            (
                crate::dependency::binary::Platform::Windows,
                crate::dependency::binary::Architecture::X86_64,
            ) => "surfpool-windows-x86_64.zip",
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported platform/architecture: {actual_platform:?}-{actual_arch:?}"
                ));
            }
        };

        // Use specific version (v0.10.8) and correct repository
        let download_url =
            format!("https://github.com/txtx/surfpool/releases/download/v0.10.8/{filename}");

        info!("Downloading from: {}", download_url);

        // Download the file
        let response = self.client.get(&download_url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to download surfpool: HTTP {}",
                response.status()
            ));
        }

        let bytes = response.bytes().await?;
        let cache_path = PathBuf::from(&self.cache_dir);

        // Ensure cache directory exists
        fs::create_dir_all(&cache_path).await?;

        // Extract the archive
        let binary_path = if filename.ends_with(".tar.gz") {
            self.extract_tar_gz(&bytes, &cache_path).await?
        } else if filename.ends_with(".zip") {
            self.extract_zip(&bytes, &cache_path).await?
        } else {
            // Direct binary
            let binary_path = cache_path.join("surfpool");
            fs::write(&binary_path, &bytes).await?;
            binary_path
        };

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path).await?.permissions();
            perms.set_mode(perms.mode() | 0o755);
            fs::set_permissions(&binary_path, perms).await?;
        }

        info!("Downloaded surfpool to: {}", binary_path.display());
        Ok(binary_path)
    }

    /// Extract tar.gz archive
    async fn extract_tar_gz(&self, bytes: &[u8], extract_dir: &Path) -> Result<PathBuf> {
        let cursor = std::io::Cursor::new(bytes);
        let decoder = GzDecoder::new(cursor);
        let mut archive = Archive::new(decoder);

        let mut binary_path = None;

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            if let Some(filename) = path.file_name() {
                if filename == "surfpool" || filename == "surfpool.exe" {
                    let output_path = extract_dir.join(filename);
                    let _output_file = fs::File::create(&output_path).await?;
                    let mut contents = Vec::new();
                    entry.read_to_end(&mut contents)?;
                    fs::write(&output_path, contents).await?;
                    binary_path = Some(output_path);
                    break;
                }
            }
        }

        binary_path.ok_or_else(|| anyhow::anyhow!("No surfpool binary found in archive"))
    }

    /// Extract zip archive
    async fn extract_zip(&self, bytes: &[u8], extract_dir: &Path) -> Result<PathBuf> {
        use zip::ZipArchive;

        let cursor = std::io::Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let path = file
                .enclosed_name()
                .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?;

            if let Some(filename) = path.file_name() {
                if filename == "surfpool.exe" {
                    let output_path = extract_dir.join(filename);
                    let _output_file = fs::File::create(&output_path).await?;
                    let mut contents = Vec::new();
                    file.read_to_end(&mut contents)?;
                    fs::write(&output_path, contents).await?;
                    return Ok(output_path);
                }
            }
        }

        Err(anyhow::anyhow!("No surfpool.exe found in zip archive"))
    }

    /// Check if a binary is cached and valid
    pub async fn is_cached_binary_valid(&self, binary_name: &str) -> Result<bool> {
        let binary_path = PathBuf::from(&self.cache_dir).join(binary_name);

        if !binary_path.exists() {
            return Ok(false);
        }

        // Check if binary is executable
        if cfg!(unix) {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&binary_path).await?;
            let perms = metadata.permissions();
            Ok(perms.mode() & 0o111 != 0)
        } else {
            Ok(true) // Windows doesn't have executable permissions
        }
    }

    /// Clean old binaries from cache
    pub async fn cleanup_old_binaries(&self, max_age_days: u64) -> Result<usize> {
        let cache_path = PathBuf::from(&self.cache_dir);
        if !cache_path.exists() {
            return Ok(0);
        }

        let mut cleaned_count = 0;
        let mut entries = fs::read_dir(&cache_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path).await {
                    if let Ok(modified) = metadata.modified() {
                        let age = modified.elapsed().unwrap_or_default();
                        if age.as_secs() > max_age_days * 24 * 60 * 60 {
                            if let Err(e) = fs::remove_file(&path).await {
                                warn!("Failed to remove old binary {}: {}", path.display(), e);
                            } else {
                                info!("Removed old binary: {}", path.display());
                                cleaned_count += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(cleaned_count)
    }

    /// Get binary information
    pub fn get_binary_info(&self, name: &str, version: &Version) -> BinaryInfo {
        let platform = Platform::current();
        let arch = crate::dependency::binary::Architecture::current();

        BinaryInfo::new(name.to_string(), version.clone(), platform, arch)
            .with_path(PathBuf::from(&self.cache_dir).join(name))
            .with_cached(true)
    }

    /// Detect actual OS platform
    pub fn detect_os_platform(&self) -> Platform {
        match std::env::consts::OS {
            "linux" => Platform::Linux,
            "macos" => Platform::MacOS,
            "windows" => Platform::Windows,
            _other => {
                // Fallback to platform detection via binary module
                Platform::current()
            }
        }
    }

    /// Detect actual OS architecture
    pub fn detect_os_architecture(&self) -> crate::dependency::binary::Architecture {
        match std::env::consts::ARCH {
            "x86_64" => crate::dependency::binary::Architecture::X86_64,
            "aarch64" => crate::dependency::binary::Architecture::Aarch64,
            _other => {
                // Fallback to architecture detection via binary module
                crate::dependency::binary::Architecture::current()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_binary_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache").to_string_lossy().to_string();
        let install_dir = temp_dir
            .path()
            .join("install")
            .to_string_lossy()
            .to_string();

        let manager = BinaryManager::new(cache_dir.clone(), install_dir, true);

        // BinaryManager doesn't create directories until needed
        // Test that the manager has the correct cache directory path
        assert_eq!(manager.cache_dir, cache_dir);
    }

    #[tokio::test]
    async fn test_is_cached_binary_valid() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache").to_string_lossy().to_string();
        let install_dir = temp_dir
            .path()
            .join("install")
            .to_string_lossy()
            .to_string();

        let manager = BinaryManager::new(cache_dir, install_dir, true);

        // Test with non-existent binary
        let result = manager.is_cached_binary_valid("nonexistent").await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_cleanup_old_binaries() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("cache").to_string_lossy().to_string();
        let install_dir = temp_dir
            .path()
            .join("install")
            .to_string_lossy()
            .to_string();

        let manager = BinaryManager::new(cache_dir, install_dir, true);

        // Cleanup empty cache should return 0
        let cleaned = manager.cleanup_old_binaries(1).await.unwrap();
        assert_eq!(cleaned, 0);
    }
}
