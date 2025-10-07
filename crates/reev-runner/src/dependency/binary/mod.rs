//! Binary management for surfpool and other dependencies
//!
//! This module handles downloading, caching, and managing pre-built binaries
//! for external dependencies like surfpool, with fallback to building from source.

use sha2::Digest;

pub mod binary_manager;
pub mod platform;
pub mod version;

pub use binary_manager::BinaryManager;
pub use platform::{Architecture, Platform};
pub use version::Version;

use anyhow::{Context, Result};
use std::{os::unix::fs::PermissionsExt, path::PathBuf};
use tracing::{info, warn};

/// Default cache directory for binaries
pub const DEFAULT_CACHE_DIR: &str = ".surfpool/cache";

/// Default install directory for binaries
pub const DEFAULT_INSTALL_DIR: &str = ".surfpool/installs";

/// Binary information
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    pub name: String,
    pub version: Version,
    pub platform: Platform,
    pub architecture: Architecture,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
    pub path: PathBuf,
    pub is_cached: bool,
}

impl BinaryInfo {
    pub fn new(
        name: String,
        version: Version,
        platform: Platform,
        architecture: Architecture,
    ) -> Self {
        Self {
            name,
            version,
            platform,
            architecture,
            download_url: None,
            checksum: None,
            path: PathBuf::new(),
            is_cached: false,
        }
    }

    pub fn with_download_url(mut self, url: String) -> Self {
        self.download_url = Some(url);
        self
    }

    pub fn with_checksum(mut self, checksum: String) -> Self {
        self.checksum = Some(checksum);
        self
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    pub fn with_cached(mut self, is_cached: bool) -> Self {
        self.is_cached = is_cached;
        self
    }

    /// Get the binary filename for the current platform
    pub fn filename(&self) -> String {
        match self.platform {
            Platform::Linux => format!("{}-{}-{}", self.name, self.version, "linux-x86_64"),
            Platform::MacOS => format!("{}-{}-{}", self.name, self.version, "darwin-x86_64"),
            Platform::Windows => format!("{}-{}-{}.exe", self.name, self.version, "windows-x86_64"),
        }
    }

    /// Get the expected binary path in cache
    pub fn cache_path(&self, cache_dir: &str) -> PathBuf {
        PathBuf::from(cache_dir).join(self.filename())
    }

    /// Get the expected binary path in installs
    pub fn install_path(&self, install_dir: &str) -> PathBuf {
        PathBuf::from(install_dir)
            .join(format!("{}-{}", self.name, self.version))
            .join(if self.platform == Platform::Windows {
                format!("{}.exe", self.name)
            } else {
                self.name.clone()
            })
    }

    /// Check if the binary exists and is executable
    pub fn is_executable(&self) -> bool {
        self.path.exists()
            && (cfg!(unix)
                && std::fs::metadata(&self.path)
                    .map(|m| m.permissions().mode() & 0o111 != 0)
                    .unwrap_or(false)
                || cfg!(windows))
    }

    /// Make the binary executable (Unix only)
    pub fn make_executable(&self) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut perms = std::fs::metadata(&self.path)
                .context("Failed to get binary permissions")?
                .permissions();
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(&self.path, perms)
                .context("Failed to set binary permissions")?;
        }

        Ok(())
    }
}

/// Result of binary acquisition
#[derive(Debug)]
pub enum BinaryAcquisitionResult {
    /// Binary was found in cache
    Cached(PathBuf),
    /// Binary was downloaded and cached
    Downloaded(PathBuf),
    /// Binary was built from source
    Built(PathBuf),
    /// Binary was found already installed
    Existing(PathBuf),
}

impl BinaryAcquisitionResult {
    pub fn path(&self) -> &PathBuf {
        match self {
            BinaryAcquisitionResult::Cached(path) => path,
            BinaryAcquisitionResult::Downloaded(path) => path,
            BinaryAcquisitionResult::Built(path) => path,
            BinaryAcquisitionResult::Existing(path) => path,
        }
    }

    pub fn was_cached(&self) -> bool {
        matches!(self, BinaryAcquisitionResult::Cached(_))
    }

    pub fn was_downloaded(&self) -> bool {
        matches!(self, BinaryAcquisitionResult::Downloaded(_))
    }

    pub fn was_built(&self) -> bool {
        matches!(self, BinaryAcquisitionResult::Built(_))
    }
}

/// Error types for binary management
#[derive(Debug, thiserror::Error)]
pub enum BinaryError {
    #[error("Failed to download binary from {url}: {source}")]
    DownloadError { url: String, source: anyhow::Error },

    #[error("Failed to extract binary archive: {source}")]
    ExtractionError { source: anyhow::Error },

    #[error("Binary checksum verification failed: expected {expected}, got {actual}")]
    ChecksumError { expected: String, actual: String },

    #[error("Failed to build binary from source: {source}")]
    BuildError { source: anyhow::Error },

    #[error("Unsupported platform: {platform} {architecture}")]
    UnsupportedPlatform {
        platform: String,
        architecture: String,
    },

    #[error("Binary not found for {name} {version} on {platform} {architecture}")]
    BinaryNotFound {
        name: String,
        version: String,
        platform: String,
        architecture: String,
    },

    #[error("Cache directory error: {source}")]
    CacheError { source: anyhow::Error },

    #[error("Permission denied accessing binary at {path}")]
    PermissionError { path: String },
}

/// Ensure cache directory exists
pub fn ensure_cache_dir(cache_dir: &str) -> Result<PathBuf, BinaryError> {
    let path = PathBuf::from(cache_dir);

    if !path.exists() {
        std::fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create cache directory: {}", path.display()))
            .map_err(|e| BinaryError::CacheError { source: e })?;
    }

    Ok(path)
}

/// Ensure install directory exists
pub fn ensure_install_dir(install_dir: &str) -> Result<PathBuf, BinaryError> {
    let path = PathBuf::from(install_dir);

    if !path.exists() {
        std::fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create install directory: {}", path.display()))
            .map_err(|e| BinaryError::CacheError { source: e })?;
    }

    Ok(path)
}

/// Clean old binaries from cache directory
pub fn cleanup_old_binaries(cache_dir: &str, max_age_days: u64) -> Result<usize> {
    let cache_path = PathBuf::from(cache_dir);
    if !cache_path.exists() {
        return Ok(0);
    }

    let mut cleaned_count = 0;
    let cutoff_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (max_age_days * 24 * 60 * 60);

    for entry in std::fs::read_dir(&cache_path)
        .with_context(|| format!("Failed to read cache directory: {}", cache_path.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    if modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        < cutoff_time
                    {
                        if let Err(e) = std::fs::remove_file(&path) {
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

/// Get binary size in bytes
pub fn get_binary_size(path: &PathBuf) -> Result<u64> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("Failed to get metadata for binary: {}", path.display()))?;
    Ok(metadata.len())
}

/// Verify binary checksum (SHA-256)
pub fn verify_checksum(path: &PathBuf, expected_checksum: &str) -> Result<bool> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).with_context(|| {
        format!(
            "Failed to open binary for checksum verification: {}",
            path.display()
        )
    })?;

    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .with_context(|| format!("Failed to read binary for checksum: {}", path.display()))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    let actual_checksum = format!("{result:x}");

    Ok(actual_checksum == expected_checksum)
}
