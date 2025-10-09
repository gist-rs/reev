//! Version management for binary dependencies

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Version representation for binary dependencies
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Option<String>,
    pub build: Option<String>,
}

impl Version {
    /// Create a new version
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
            build: None,
        }
    }

    /// Create a version with pre-release identifier
    pub fn with_pre(mut self, pre: String) -> Self {
        self.pre = Some(pre);
        self
    }

    /// Create a version with build metadata
    pub fn with_build(mut self, build: String) -> Self {
        self.build = Some(build);
        self
    }

    /// Check if this is a pre-release version
    pub fn is_prerelease(&self) -> bool {
        self.pre.is_some()
    }

    /// Check if this is a stable release
    pub fn is_stable(&self) -> bool {
        self.pre.is_none()
    }

    /// Get the version as a string without pre-release or build metadata
    pub fn core_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }

    /// Get the full version string
    pub fn full_string(&self) -> String {
        let mut result = self.core_string();

        if let Some(pre) = &self.pre {
            result.push('-');
            result.push_str(pre);
        }

        if let Some(build) = &self.build {
            result.push('+');
            result.push_str(build);
        }

        result
    }

    /// Check if this version satisfies a requirement
    pub fn satisfies(&self, requirement: &VersionRequirement) -> bool {
        requirement.matches(self)
    }

    /// Get the next major version
    pub fn next_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    /// Get the next minor version
    pub fn next_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    /// Get the next patch version
    pub fn next_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_string())
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '+');
        let main_part = parts.next().ok_or(VersionError::InvalidFormat)?;
        let build_part = parts.next();

        let mut parts = main_part.splitn(2, '-');
        let version_part = parts.next().ok_or(VersionError::InvalidFormat)?;
        let pre_part = parts.next();

        let version_components: Vec<&str> = version_part.split('.').collect();
        if version_components.len() != 3 {
            return Err(VersionError::InvalidFormat);
        }

        let major = version_components[0]
            .parse()
            .map_err(|_| VersionError::InvalidNumber)?;
        let minor = version_components[1]
            .parse()
            .map_err(|_| VersionError::InvalidNumber)?;
        let patch = version_components[2]
            .parse()
            .map_err(|_| VersionError::InvalidNumber)?;

        let mut version = Self::new(major, minor, patch);

        if let Some(pre) = pre_part {
            version = version.with_pre(pre.to_string());
        }

        if let Some(build) = build_part {
            version = version.with_build(build.to_string());
        }

        Ok(version)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major, minor, patch
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            other => return other,
        }

        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            other => return other,
        }

        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            other => return other,
        }

        // Compare pre-release identifiers
        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater, // Stable > pre-release
            (Some(_), None) => Ordering::Less,    // Pre-release < stable
            (Some(self_pre), Some(other_pre)) => {
                // Compare pre-release identifiers lexicographically
                self_pre.cmp(other_pre)
            }
        }
    }
}

/// Version requirement specifications
#[derive(Debug, Clone)]
pub enum VersionRequirement {
    /// Exact version match
    Exact(Version),
    /// Minimum version (>=)
    Minimum(Version),
    /// Maximum version (<=)
    Maximum(Version),
    /// Version range (>= min, <= max)
    Range { min: Version, max: Version },
    /// Compatible version (^)
    Compatible(Version),
    /// Any version
    Any,
}

impl VersionRequirement {
    /// Check if a version satisfies this requirement
    pub fn matches(&self, version: &Version) -> bool {
        match self {
            VersionRequirement::Exact(req) => version == req,
            VersionRequirement::Minimum(req) => version >= req,
            VersionRequirement::Maximum(req) => version <= req,
            VersionRequirement::Range { min, max } => version >= min && version <= max,
            VersionRequirement::Compatible(req) => {
                // Compatible means same major version and >= required version
                version.major == req.major && version >= req
            }
            VersionRequirement::Any => true,
        }
    }

    /// Parse a version requirement from a string
    pub fn parse(s: &str) -> Result<Self, VersionError> {
        let s = s.trim();

        if s == "*" || s == "any" {
            return Ok(VersionRequirement::Any);
        }

        if let Some(version_str) = s.strip_prefix('^') {
            let version = version_str.parse()?;
            return Ok(VersionRequirement::Compatible(version));
        }

        if let Some(version_str) = s.strip_prefix(">=") {
            let version = version_str.parse()?;
            return Ok(VersionRequirement::Minimum(version));
        } else if let Some(version_str) = s.strip_prefix('>') {
            let version: Version = version_str.parse()?;
            return Ok(VersionRequirement::Minimum(version.next_patch()));
        }

        if let Some(version_str) = s.strip_prefix("<=") {
            let version: Version = version_str.parse()?;
            return Ok(VersionRequirement::Maximum(version));
        } else if let Some(version_str) = s.strip_prefix('<') {
            let version: Version = version_str.parse()?;
            return Ok(VersionRequirement::Maximum(version.previous_patch()));
        }

        // Check for range format "min - max" or "min,max"
        if s.contains('-') || s.contains(',') {
            let parts: Vec<&str> = if s.contains('-') {
                s.split('-').collect()
            } else {
                s.split(',').collect()
            };

            if parts.len() == 2 {
                let min = parts[0].trim().parse()?;
                let max = parts[1].trim().parse()?;
                return Ok(VersionRequirement::Range { min, max });
            }
        }

        // Default to exact version
        let version = s.parse()?;
        Ok(VersionRequirement::Exact(version))
    }
}

impl Version {
    /// Get the previous patch version (for upper bounds)
    fn previous_patch(&self) -> Self {
        if self.patch > 0 {
            Self::new(self.major, self.minor, self.patch - 1)
        } else if self.minor > 0 {
            Self::new(self.major, self.minor - 1, 999)
        } else if self.major > 0 {
            Self::new(self.major - 1, 999, 999)
        } else {
            Self::new(0, 0, 0)
        }
    }
}

/// Version-related errors
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Invalid version format")]
    InvalidFormat,

    #[error("Invalid version number")]
    InvalidNumber,

    #[error("Invalid version requirement: {0}")]
    InvalidRequirement(String),
}

/// Version comparison utilities
pub struct VersionUtils;

impl VersionUtils {
    /// Find the latest version that satisfies a requirement
    pub fn find_latest_matching<'a>(
        versions: &'a [Version],
        requirement: &'a VersionRequirement,
    ) -> Option<&'a Version> {
        versions.iter().filter(|v| requirement.matches(v)).max()
    }

    /// Find the minimum version that satisfies a requirement
    pub fn find_minimum_matching<'a>(
        versions: &'a [Version],
        requirement: &'a VersionRequirement,
    ) -> Option<&'a Version> {
        versions.iter().filter(|v| requirement.matches(v)).min()
    }

    /// Sort versions in ascending order
    pub fn sort_versions(versions: &mut [Version]) {
        versions.sort();
    }

    /// Check if a version is newer than another
    pub fn is_newer(current: &Version, other: &Version) -> bool {
        current > other
    }

    /// Get the latest stable version from a list
    pub fn latest_stable(versions: &[Version]) -> Option<&Version> {
        versions.iter().filter(|v| v.is_stable()).max()
    }

    /// Get the latest version (including pre-releases) from a list
    pub fn latest_any(versions: &[Version]) -> Option<&Version> {
        versions.iter().max()
    }
}
