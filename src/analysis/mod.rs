//! Analysis module for CodeScope.
//!
//! This module provides static analysis tools for JavaScript/TypeScript projects,
//! including import analysis to detect which dependency exports are actually used.
//!
//! # Features
//!
//! - **Import Detection**: Parse JS/TS files to extract import statements
//! - **Usage Tracking**: Track which symbols are imported from each package
//! - **Utilization Analysis**: Calculate how much of a dependency is being used
//!
//! # Example
//!
//! ```ignore
//! use codescope::analysis::{ImportAnalyzer, ImportInfo};
//! use std::path::Path;
//!
//! let analyzer = ImportAnalyzer::new();
//! let imports = analyzer.analyze_file(Path::new("src/app.ts")).unwrap();
//!
//! for import in &imports {
//!     println!("{}: {:?}", import.package_name, import.imported_symbols);
//! }
//! ```

pub mod exports;

// Re-export commonly used types for convenience
pub use exports::{
    AnalysisError, AnalysisResult, FileType, ImportAnalyzer, ImportInfo, ImportStyle, ImportUsage,
    ProjectImports,
};
