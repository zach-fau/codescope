//! Source code analysis module for CodeScope.
//!
//! This module provides tools for analyzing JavaScript/TypeScript source code
//! to track import usage and identify unused exports from dependencies.
//!
//! # Features
//!
//! - Parse ES6 `import` statements (default, named, namespace imports)
//! - Parse CommonJS `require()` calls
//! - Track which exports from each dependency are actually used
//! - Calculate utilization percentage per dependency
//! - Flag low-utilization dependencies
//!
//! # Example
//!
//! ```ignore
//! use std::path::Path;
//! use codescope::analysis::{ImportAnalyzer, analyze_project_imports};
//!
//! // Analyze a single file
//! let analyzer = ImportAnalyzer::new();
//! let imports = analyzer.analyze_file(Path::new("src/index.js"))?;
//!
//! for import in imports {
//!     println!("{}: {:?}", import.source, import.specifiers);
//! }
//!
//! // Analyze entire project
//! let usage = analyze_project_imports(Path::new("./src"))?;
//! for (package, info) in usage.iter() {
//!     println!("{}: {:.1}% utilized", package, info.utilization_percentage());
//! }
//! ```

pub mod exports;

// Re-export main types for convenience
pub use exports::{
    analyze_file, analyze_project_imports, Import, ImportAnalyzer, ImportKind, ImportSpecifier,
    PackageUsage, ProjectImports,
};
