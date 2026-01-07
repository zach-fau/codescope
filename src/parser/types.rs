//! Shared types for dependency parsing.
//!
//! This module defines the core data structures used to represent
//! package manifests and their dependencies across different ecosystems.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents the structure of a package.json file.
///
/// This struct mirrors the npm package.json specification,
/// capturing the essential fields needed for dependency analysis.
///
/// # Example
///
/// ```ignore
/// use codescope::parser::types::PackageJson;
/// use serde_json;
///
/// let json = r#"{"name": "my-app", "version": "1.0.0"}"#;
/// let pkg: PackageJson = serde_json::from_str(json).unwrap();
/// assert_eq!(pkg.name, Some("my-app".to_string()));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackageJson {
    /// The name of the package.
    pub name: Option<String>,

    /// The version of the package (semver format).
    pub version: Option<String>,

    /// A brief description of the package.
    pub description: Option<String>,

    /// Production dependencies required at runtime.
    pub dependencies: Option<HashMap<String, String>>,

    /// Development-only dependencies (testing, building, etc.).
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,

    /// Peer dependencies that the host package must provide.
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: Option<HashMap<String, String>>,

    /// Optional dependencies that enhance functionality if available.
    #[serde(rename = "optionalDependencies")]
    pub optional_dependencies: Option<HashMap<String, String>>,
}

impl PackageJson {
    /// Returns true if the package has any dependencies defined.
    pub fn has_dependencies(&self) -> bool {
        self.dependencies.as_ref().is_some_and(|d| !d.is_empty())
            || self
                .dev_dependencies
                .as_ref()
                .is_some_and(|d| !d.is_empty())
            || self
                .peer_dependencies
                .as_ref()
                .is_some_and(|d| !d.is_empty())
            || self
                .optional_dependencies
                .as_ref()
                .is_some_and(|d| !d.is_empty())
    }

    /// Returns the total count of all dependencies.
    pub fn dependency_count(&self) -> usize {
        self.dependencies.as_ref().map_or(0, |d| d.len())
            + self.dev_dependencies.as_ref().map_or(0, |d| d.len())
            + self.peer_dependencies.as_ref().map_or(0, |d| d.len())
            + self.optional_dependencies.as_ref().map_or(0, |d| d.len())
    }
}

/// Categorizes the type of dependency relationship.
///
/// Different dependency types have different implications for
/// bundle size, deployment, and version resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DependencyType {
    /// Production dependencies - required at runtime.
    /// These are bundled with the application.
    Production,

    /// Development dependencies - only needed during development.
    /// Not included in production builds.
    Development,

    /// Peer dependencies - expected to be provided by the consumer.
    /// Used for plugins and extensions.
    Peer,

    /// Optional dependencies - enhance functionality if available.
    /// Installation continues even if they fail.
    Optional,
}

impl DependencyType {
    /// Returns a short label for the dependency type.
    pub fn label(&self) -> &'static str {
        match self {
            DependencyType::Production => "prod",
            DependencyType::Development => "dev",
            DependencyType::Peer => "peer",
            DependencyType::Optional => "optional",
        }
    }

    /// Returns true if this dependency type affects production bundle size.
    pub fn affects_bundle_size(&self) -> bool {
        matches!(self, DependencyType::Production | DependencyType::Optional)
    }
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            DependencyType::Production => "production",
            DependencyType::Development => "development",
            DependencyType::Peer => "peer",
            DependencyType::Optional => "optional",
        };
        write!(f, "{}", s)
    }
}

/// Represents a single dependency with its metadata.
///
/// This is the normalized form used throughout CodeScope,
/// abstracting away the differences between package managers.
#[derive(Debug, Clone)]
pub struct Dependency {
    /// The package name (e.g., "react", "lodash").
    pub name: String,

    /// The version specifier (e.g., "^18.0.0", "~1.2.3").
    pub version: String,

    /// The category of this dependency.
    pub dep_type: DependencyType,
}

impl Dependency {
    /// Creates a new Dependency instance.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        dep_type: DependencyType,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            dep_type,
        }
    }

    /// Returns true if this is a production dependency.
    pub fn is_production(&self) -> bool {
        self.dep_type == DependencyType::Production
    }

    /// Returns true if this is a development dependency.
    pub fn is_development(&self) -> bool {
        self.dep_type == DependencyType::Development
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{} ({})", self.name, self.version, self.dep_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_type_label() {
        assert_eq!(DependencyType::Production.label(), "prod");
        assert_eq!(DependencyType::Development.label(), "dev");
        assert_eq!(DependencyType::Peer.label(), "peer");
        assert_eq!(DependencyType::Optional.label(), "optional");
    }

    #[test]
    fn test_dependency_type_affects_bundle_size() {
        assert!(DependencyType::Production.affects_bundle_size());
        assert!(!DependencyType::Development.affects_bundle_size());
        assert!(!DependencyType::Peer.affects_bundle_size());
        assert!(DependencyType::Optional.affects_bundle_size());
    }

    #[test]
    fn test_dependency_new() {
        let dep = Dependency::new("react", "^18.0.0", DependencyType::Production);
        assert_eq!(dep.name, "react");
        assert_eq!(dep.version, "^18.0.0");
        assert_eq!(dep.dep_type, DependencyType::Production);
    }

    #[test]
    fn test_dependency_display() {
        let dep = Dependency::new("lodash", "~4.17.21", DependencyType::Development);
        assert_eq!(format!("{}", dep), "lodash@~4.17.21 (development)");
    }

    #[test]
    fn test_package_json_default() {
        let pkg = PackageJson::default();
        assert!(pkg.name.is_none());
        assert!(!pkg.has_dependencies());
        assert_eq!(pkg.dependency_count(), 0);
    }

    #[test]
    fn test_package_json_has_dependencies() {
        let mut pkg = PackageJson::default();
        assert!(!pkg.has_dependencies());

        let mut deps = HashMap::new();
        deps.insert("react".to_string(), "^18.0.0".to_string());
        pkg.dependencies = Some(deps);

        assert!(pkg.has_dependencies());
        assert_eq!(pkg.dependency_count(), 1);
    }
}
