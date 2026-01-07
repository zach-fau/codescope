//! Parser for npm package.json files.
//!
//! This module provides functionality to parse package.json files
//! and extract dependency information for analysis.

use std::fs;
use std::path::Path;

use super::types::{Dependency, DependencyType, PackageJson};

/// Errors that can occur during package.json parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Failed to read the file from disk.
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),

    /// Failed to parse JSON content.
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),

    /// The package.json structure is invalid or missing required fields.
    #[error("Invalid package.json: {0}")]
    InvalidPackage(String),
}

/// Result type alias for parser operations.
pub type ParseResult<T> = Result<T, ParseError>;

/// Parses a package.json file from a file path.
///
/// # Arguments
///
/// * `path` - Path to the package.json file
///
/// # Returns
///
/// A `ParseResult` containing the parsed `PackageJson` or an error.
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
/// use codescope::parser::package_json::parse_file;
///
/// let pkg = parse_file(Path::new("package.json")).unwrap();
/// println!("Package: {:?}", pkg.name);
/// ```
pub fn parse_file(path: &Path) -> ParseResult<PackageJson> {
    let content = fs::read_to_string(path)?;
    parse_str(&content)
}

/// Parses a package.json from a string.
///
/// # Arguments
///
/// * `content` - JSON string content of the package.json
///
/// # Returns
///
/// A `ParseResult` containing the parsed `PackageJson` or an error.
///
/// # Example
///
/// ```
/// use codescope::parser::package_json::parse_str;
///
/// let json = r#"{"name": "my-app", "version": "1.0.0"}"#;
/// let pkg = parse_str(json).unwrap();
/// assert_eq!(pkg.name, Some("my-app".to_string()));
/// ```
pub fn parse_str(content: &str) -> ParseResult<PackageJson> {
    let pkg: PackageJson = serde_json::from_str(content)?;
    Ok(pkg)
}

/// Validates a parsed PackageJson structure.
///
/// Checks for common issues and ensures the package has meaningful content.
///
/// # Arguments
///
/// * `pkg` - Reference to the PackageJson to validate
///
/// # Returns
///
/// A `ParseResult` with `()` on success or an error describing the issue.
pub fn validate(pkg: &PackageJson) -> ParseResult<()> {
    // A package.json should have at least a name or dependencies
    if pkg.name.is_none() && !pkg.has_dependencies() {
        return Err(ParseError::InvalidPackage(
            "package.json has no name and no dependencies".to_string(),
        ));
    }
    Ok(())
}

/// Extracts all dependencies from a PackageJson into a normalized list.
///
/// This function collects dependencies from all categories (production,
/// development, peer, optional) and returns them as a flat list with
/// their types tagged.
///
/// # Arguments
///
/// * `pkg` - Reference to the PackageJson to extract from
///
/// # Returns
///
/// A `Vec<Dependency>` containing all dependencies with their types.
///
/// # Example
///
/// ```
/// use codescope::parser::package_json::{parse_str, extract_dependencies};
/// use codescope::parser::types::DependencyType;
///
/// let json = r#"{
///     "name": "my-app",
///     "dependencies": {"react": "^18.0.0"},
///     "devDependencies": {"typescript": "^5.0.0"}
/// }"#;
///
/// let pkg = parse_str(json).unwrap();
/// let deps = extract_dependencies(&pkg);
///
/// assert_eq!(deps.len(), 2);
/// assert!(deps.iter().any(|d| d.name == "react" && d.dep_type == DependencyType::Production));
/// assert!(deps.iter().any(|d| d.name == "typescript" && d.dep_type == DependencyType::Development));
/// ```
pub fn extract_dependencies(pkg: &PackageJson) -> Vec<Dependency> {
    let mut deps = Vec::new();

    // Extract production dependencies
    if let Some(ref dependencies) = pkg.dependencies {
        for (name, version) in dependencies {
            deps.push(Dependency::new(name, version, DependencyType::Production));
        }
    }

    // Extract development dependencies
    if let Some(ref dev_dependencies) = pkg.dev_dependencies {
        for (name, version) in dev_dependencies {
            deps.push(Dependency::new(name, version, DependencyType::Development));
        }
    }

    // Extract peer dependencies
    if let Some(ref peer_dependencies) = pkg.peer_dependencies {
        for (name, version) in peer_dependencies {
            deps.push(Dependency::new(name, version, DependencyType::Peer));
        }
    }

    // Extract optional dependencies
    if let Some(ref optional_dependencies) = pkg.optional_dependencies {
        for (name, version) in optional_dependencies {
            deps.push(Dependency::new(name, version, DependencyType::Optional));
        }
    }

    deps
}

/// Extracts only production dependencies from a PackageJson.
///
/// This is useful for bundle size analysis where only production
/// dependencies matter.
///
/// # Arguments
///
/// * `pkg` - Reference to the PackageJson to extract from
///
/// # Returns
///
/// A `Vec<Dependency>` containing only production dependencies.
pub fn extract_production_dependencies(pkg: &PackageJson) -> Vec<Dependency> {
    extract_dependencies(pkg)
        .into_iter()
        .filter(|d| d.dep_type == DependencyType::Production)
        .collect()
}

/// Groups dependencies by their type.
///
/// # Arguments
///
/// * `deps` - Slice of dependencies to group
///
/// # Returns
///
/// A tuple of four vectors: (production, development, peer, optional)
pub fn group_by_type(
    deps: &[Dependency],
) -> (
    Vec<&Dependency>,
    Vec<&Dependency>,
    Vec<&Dependency>,
    Vec<&Dependency>,
) {
    let mut prod = Vec::new();
    let mut dev = Vec::new();
    let mut peer = Vec::new();
    let mut optional = Vec::new();

    for dep in deps {
        match dep.dep_type {
            DependencyType::Production => prod.push(dep),
            DependencyType::Development => dev.push(dep),
            DependencyType::Peer => peer.push(dep),
            DependencyType::Optional => optional.push(dep),
        }
    }

    (prod, dev, peer, optional)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PACKAGE_JSON: &str = r#"{
        "name": "test-app",
        "version": "1.0.0",
        "description": "A test application",
        "dependencies": {
            "react": "^18.2.0",
            "react-dom": "^18.2.0",
            "lodash": "^4.17.21"
        },
        "devDependencies": {
            "typescript": "^5.0.0",
            "jest": "^29.0.0"
        },
        "peerDependencies": {
            "react": ">=16.8.0"
        },
        "optionalDependencies": {
            "fsevents": "^2.3.0"
        }
    }"#;

    #[test]
    fn test_parse_str_valid() {
        let pkg = parse_str(SAMPLE_PACKAGE_JSON).unwrap();

        assert_eq!(pkg.name, Some("test-app".to_string()));
        assert_eq!(pkg.version, Some("1.0.0".to_string()));
        assert_eq!(pkg.description, Some("A test application".to_string()));
    }

    #[test]
    fn test_parse_str_minimal() {
        let json = r#"{"name": "minimal"}"#;
        let pkg = parse_str(json).unwrap();

        assert_eq!(pkg.name, Some("minimal".to_string()));
        assert!(pkg.dependencies.is_none());
    }

    #[test]
    fn test_parse_str_empty_object() {
        let json = "{}";
        let pkg = parse_str(json).unwrap();

        assert!(pkg.name.is_none());
        assert!(pkg.version.is_none());
    }

    #[test]
    fn test_parse_str_invalid_json() {
        let json = "{ invalid json }";
        let result = parse_str(json);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::JsonError(_)));
    }

    #[test]
    fn test_validate_valid_package() {
        let pkg = parse_str(SAMPLE_PACKAGE_JSON).unwrap();
        assert!(validate(&pkg).is_ok());
    }

    #[test]
    fn test_validate_name_only() {
        let json = r#"{"name": "just-a-name"}"#;
        let pkg = parse_str(json).unwrap();
        assert!(validate(&pkg).is_ok());
    }

    #[test]
    fn test_validate_deps_only() {
        let json = r#"{"dependencies": {"react": "^18.0.0"}}"#;
        let pkg = parse_str(json).unwrap();
        assert!(validate(&pkg).is_ok());
    }

    #[test]
    fn test_validate_empty_invalid() {
        let pkg = parse_str("{}").unwrap();
        let result = validate(&pkg);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::InvalidPackage(_)));
    }

    #[test]
    fn test_extract_dependencies_all_types() {
        let pkg = parse_str(SAMPLE_PACKAGE_JSON).unwrap();
        let deps = extract_dependencies(&pkg);

        // 3 prod + 2 dev + 1 peer + 1 optional = 7
        assert_eq!(deps.len(), 7);

        // Check production deps
        let prod_count = deps
            .iter()
            .filter(|d| d.dep_type == DependencyType::Production)
            .count();
        assert_eq!(prod_count, 3);

        // Check dev deps
        let dev_count = deps
            .iter()
            .filter(|d| d.dep_type == DependencyType::Development)
            .count();
        assert_eq!(dev_count, 2);

        // Check peer deps
        let peer_count = deps
            .iter()
            .filter(|d| d.dep_type == DependencyType::Peer)
            .count();
        assert_eq!(peer_count, 1);

        // Check optional deps
        let optional_count = deps
            .iter()
            .filter(|d| d.dep_type == DependencyType::Optional)
            .count();
        assert_eq!(optional_count, 1);
    }

    #[test]
    fn test_extract_dependencies_specific_values() {
        let pkg = parse_str(SAMPLE_PACKAGE_JSON).unwrap();
        let deps = extract_dependencies(&pkg);

        // Find react dependency
        let react = deps
            .iter()
            .find(|d| d.name == "react" && d.dep_type == DependencyType::Production);
        assert!(react.is_some());
        assert_eq!(react.unwrap().version, "^18.2.0");

        // Find typescript dependency
        let typescript = deps.iter().find(|d| d.name == "typescript");
        assert!(typescript.is_some());
        assert_eq!(typescript.unwrap().dep_type, DependencyType::Development);
    }

    #[test]
    fn test_extract_production_dependencies() {
        let pkg = parse_str(SAMPLE_PACKAGE_JSON).unwrap();
        let deps = extract_production_dependencies(&pkg);

        assert_eq!(deps.len(), 3);
        assert!(deps
            .iter()
            .all(|d| d.dep_type == DependencyType::Production));
    }

    #[test]
    fn test_group_by_type() {
        let pkg = parse_str(SAMPLE_PACKAGE_JSON).unwrap();
        let deps = extract_dependencies(&pkg);
        let (prod, dev, peer, optional) = group_by_type(&deps);

        assert_eq!(prod.len(), 3);
        assert_eq!(dev.len(), 2);
        assert_eq!(peer.len(), 1);
        assert_eq!(optional.len(), 1);
    }

    #[test]
    fn test_extract_dependencies_empty() {
        let json = r#"{"name": "empty-deps"}"#;
        let pkg = parse_str(json).unwrap();
        let deps = extract_dependencies(&pkg);

        assert!(deps.is_empty());
    }

    #[test]
    fn test_parse_str_with_extra_fields() {
        // package.json often has many other fields; ensure we ignore them gracefully
        let json = r#"{
            "name": "with-extras",
            "version": "1.0.0",
            "scripts": {"build": "tsc"},
            "author": "Test Author",
            "license": "MIT",
            "repository": {"type": "git", "url": "https://example.com"},
            "dependencies": {"express": "^4.18.0"}
        }"#;

        let pkg = parse_str(json).unwrap();
        assert_eq!(pkg.name, Some("with-extras".to_string()));
        assert!(pkg.dependencies.is_some());
        assert_eq!(pkg.dependencies.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_parse_error_display() {
        let io_err = ParseError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(io_err.to_string().contains("Failed to read file"));

        let invalid_err = ParseError::InvalidPackage("missing name".to_string());
        assert!(invalid_err.to_string().contains("Invalid package.json"));
    }
}
