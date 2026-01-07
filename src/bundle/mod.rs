//! Bundle size analysis module
//!
//! This module provides tools for parsing and analyzing JavaScript bundle
//! output files (e.g., webpack stats.json) to determine the size contribution
//! of each npm package in the final bundle.
//!
//! # Supported Formats
//!
//! - **Webpack**: Parse `stats.json` output from webpack builds or webpack-bundle-analyzer
//! - **Vite**: (Future) Parse vite-bundle-visualizer output
//!
//! # Example
//!
//! ```ignore
//! use codescope::bundle::{WebpackStats, BundleAnalysis};
//!
//! // Parse webpack stats
//! let stats = WebpackStats::from_file("stats.json")?;
//!
//! // Analyze bundle sizes
//! let analysis = stats.analyze();
//!
//! // Get packages sorted by size
//! for pkg in analysis.packages_by_size() {
//!     println!("{}: {} bytes", pkg.name, pkg.total_size);
//! }
//! ```

pub mod webpack;

// Re-export main types for convenience
pub use webpack::{
    extract_package_name, format_size, BundleAnalysis, PackageBundleSize, WebpackAsset,
    WebpackChunk, WebpackModule, WebpackStats,
};
