//! Platform detection for binary management

use std::env;

/// Operating system platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Linux,
    MacOS,
    Windows,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Self {
        match env::consts::OS {
            "linux" => Platform::Linux,
            "macos" => Platform::MacOS,
            "windows" => Platform::Windows,
            other => panic!("Unsupported platform: {other}"),
        }
    }

    /// Get platform name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOS => "darwin",
            Platform::Windows => "windows",
        }
    }

    /// Get platform name for binary naming
    pub fn binary_name(&self) -> &'static str {
        match self {
            Platform::Linux => "linux",
            Platform::MacOS => "darwin",
            Platform::Windows => "windows",
        }
    }

    /// Check if the platform supports the current architecture
    pub fn supports_architecture(&self, arch: Architecture) -> bool {
        match (self, arch) {
            (Platform::Linux, Architecture::X86_64) => true,
            (Platform::Linux, Architecture::Aarch64) => true,
            (Platform::MacOS, Architecture::X86_64) => true,
            (Platform::MacOS, Architecture::Aarch64) => true,
            (Platform::Windows, Architecture::X86_64) => true,
            (Platform::Windows, Architecture::Aarch64) => true,
        }
    }
}

/// CPU architectures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Architecture {
    X86_64,
    Aarch64,
}

impl Architecture {
    /// Get the current architecture
    pub fn current() -> Self {
        match env::consts::ARCH {
            "x86_64" => Architecture::X86_64,
            "aarch64" => Architecture::Aarch64,
            other => panic!("Unsupported architecture: {other}"),
        }
    }

    /// Get architecture name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64 => "aarch64",
        }
    }

    /// Get architecture name for binary naming
    pub fn binary_name(&self) -> &'static str {
        match self {
            Architecture::X86_64 => "x86_64",
            Architecture::Aarch64 => "aarch64",
        }
    }
}

/// Platform and architecture combination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlatformTriple {
    pub platform: Platform,
    pub architecture: Architecture,
}

impl PlatformTriple {
    /// Get the current platform triple
    pub fn current() -> Self {
        Self {
            platform: Platform::current(),
            architecture: Architecture::current(),
        }
    }

    /// Create a new platform triple
    pub fn new(platform: Platform, architecture: Architecture) -> Self {
        Self {
            platform,
            architecture,
        }
    }

    /// Get the triple as a string (e.g., "x86_64-linux")
    pub fn as_string(&self) -> String {
        format!("{}-{}", self.architecture.as_str(), self.platform.as_str())
    }

    /// Get the triple as a string for binary naming (e.g., "linux-x86_64")
    pub fn binary_string(&self) -> String {
        format!(
            "{}-{}",
            self.platform.binary_name(),
            self.architecture.binary_name()
        )
    }

    /// Check if this is the current platform
    pub fn is_current(&self) -> bool {
        *self == Self::current()
    }

    /// Parse a platform triple string
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid platform triple: {s}"));
        }

        let platform = match parts[1] {
            "linux" => Platform::Linux,
            "darwin" => Platform::MacOS,
            "windows" => Platform::Windows,
            other => return Err(format!("Unsupported platform: {other}")),
        };

        let architecture = match parts[0] {
            "x86_64" => Architecture::X86_64,
            "aarch64" => Architecture::Aarch64,
            other => return Err(format!("Unsupported architecture: {other}")),
        };

        Ok(Self::new(platform, architecture))
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::fmt::Display for PlatformTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}
