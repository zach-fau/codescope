//! Parser module for CodeScope.
//!
//! This module provides parsers for various package manifest formats
//! used in different ecosystems (npm, Cargo, etc.).
//!
//! # Supported Formats
//!
//! - **package.json** (npm/Node.js) - Fully supported
//! - **Cargo.toml** (Rust) - Planned
//! - **go.mod** (Go) - Planned
//! - **pyproject.toml** (Python) - Planned
//!
//! # Example
//!
//! ```ignore
//! use std::path::Path;
//! use codescope::parser::{package_json, types::DependencyType};
//!
//! // Parse a package.json file
//! let pkg = package_json::parse_file(Path::new("package.json")).unwrap();
//!
//! // Extract all dependencies
//! let deps = package_json::extract_dependencies(&pkg);
//!
//! // Filter to production only
//! let prod_deps: Vec<_> = deps.iter()
//!     .filter(|d| d.dep_type == DependencyType::Production)
//!     .collect();
//!
//! println!("Found {} production dependencies", prod_deps.len());
//! ```

pub mod package_json;
pub mod types;

// Re-export commonly used types for convenience
pub use package_json::{
    extract_dependencies, extract_production_dependencies, group_by_type, parse_file, parse_str,
    validate, ParseError, ParseResult,
};

pub use types::{Dependency, DependencyType, PackageJson};
