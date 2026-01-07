//! Webpack bundle stats parser
//!
//! This module handles parsing of webpack-bundle-analyzer JSON output (stats.json)
//! to extract module sizes, chunks, and asset information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Represents a webpack stats.json file output.
///
/// This is the top-level structure produced by webpack when configured
/// with `--json` flag or webpack-bundle-analyzer.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackStats {
    /// Webpack version used for the build
    pub version: Option<String>,

    /// Build hash identifier
    pub hash: Option<String>,

    /// Build timestamp in milliseconds
    pub time: Option<u64>,

    /// Output path of the build
    pub output_path: Option<String>,

    /// List of generated assets (output files)
    #[serde(default)]
    pub assets: Vec<WebpackAsset>,

    /// List of chunks (code-split bundles)
    #[serde(default)]
    pub chunks: Vec<WebpackChunk>,

    /// List of all modules included in the build
    #[serde(default)]
    pub modules: Vec<WebpackModule>,

    /// Entry points for the application
    #[serde(default)]
    pub entrypoints: HashMap<String, WebpackEntrypoint>,

    /// Named chunk groups
    #[serde(default)]
    pub named_chunk_groups: HashMap<String, WebpackChunkGroup>,

    /// Build errors
    #[serde(default)]
    pub errors: Vec<WebpackError>,

    /// Build warnings
    #[serde(default)]
    pub warnings: Vec<WebpackWarning>,
}

/// Represents a generated asset file from webpack build.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackAsset {
    /// Asset file name
    pub name: String,

    /// Size in bytes
    pub size: u64,

    /// Chunk IDs this asset belongs to
    #[serde(default)]
    pub chunks: Vec<ChunkId>,

    /// Chunk names this asset belongs to
    #[serde(default)]
    pub chunk_names: Vec<String>,

    /// Whether this is an emitted asset
    #[serde(default)]
    pub emitted: bool,

    /// Additional asset info
    #[serde(default)]
    pub info: WebpackAssetInfo,
}

/// Additional information about an asset.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackAssetInfo {
    /// Whether the asset is immutable (can be cached indefinitely)
    #[serde(default)]
    pub immutable: bool,

    /// Whether the asset is development-only
    #[serde(default)]
    pub development: bool,

    /// Whether the asset is hot update related
    #[serde(default)]
    pub hot_module_replacement: bool,
}

/// Represents a webpack chunk (code-split bundle).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackChunk {
    /// Chunk ID (can be number or string)
    pub id: Option<ChunkId>,

    /// Chunk names
    #[serde(default)]
    pub names: Vec<String>,

    /// Total size of the chunk in bytes
    #[serde(default)]
    pub size: u64,

    /// Files generated for this chunk
    #[serde(default)]
    pub files: Vec<String>,

    /// Whether this is an entry chunk
    #[serde(default)]
    pub entry: bool,

    /// Whether this is an initial chunk
    #[serde(default)]
    pub initial: bool,

    /// Whether the chunk is rendered
    #[serde(default)]
    pub rendered: bool,

    /// Module IDs included in this chunk
    #[serde(default)]
    pub modules: Vec<WebpackModule>,

    /// Parent chunk IDs
    #[serde(default)]
    pub parents: Vec<ChunkId>,

    /// Child chunk IDs
    #[serde(default)]
    pub children: Vec<ChunkId>,

    /// Sibling chunk IDs
    #[serde(default)]
    pub siblings: Vec<ChunkId>,

    /// Origins/reasons for this chunk
    #[serde(default)]
    pub origins: Vec<ChunkOrigin>,
}

/// Origin/reason for a chunk's creation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChunkOrigin {
    /// Module that caused this chunk
    pub module: Option<String>,

    /// Module identifier
    pub module_identifier: Option<String>,

    /// Module name
    pub module_name: Option<String>,

    /// Location in the module
    pub loc: Option<String>,

    /// Request string
    pub request: Option<String>,

    /// Reasons for the chunk
    #[serde(default)]
    pub reasons: Vec<String>,
}

/// Represents a module in the webpack build.
///
/// This is the core unit for bundle size analysis, as it contains
/// the actual code that gets bundled.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackModule {
    /// Unique module identifier (full path)
    pub identifier: Option<String>,

    /// Short module name/path
    pub name: Option<String>,

    /// Module size in bytes
    #[serde(default)]
    pub size: u64,

    /// Whether the module is cacheable
    #[serde(default)]
    pub cacheable: bool,

    /// Whether the module is built
    #[serde(default)]
    pub built: bool,

    /// Whether the module is optional
    #[serde(default)]
    pub optional: bool,

    /// Module prefetch order
    pub prefetched: Option<bool>,

    /// Chunk IDs this module belongs to
    #[serde(default)]
    pub chunks: Vec<ChunkId>,

    /// Reasons why this module was included
    #[serde(default)]
    pub reasons: Vec<ModuleReason>,

    /// Assets generated by this module
    #[serde(default)]
    pub assets: Vec<String>,

    /// Source code (if included in stats)
    pub source: Option<String>,

    /// Module type (e.g., "javascript/auto")
    #[serde(rename = "type")]
    pub module_type: Option<String>,

    /// Issuer module path
    pub issuer: Option<String>,

    /// Issuer module name
    pub issuer_name: Option<String>,

    /// Nested modules (for concatenated modules)
    #[serde(default)]
    pub modules: Vec<WebpackModule>,

    /// Module depth in the dependency tree
    pub depth: Option<usize>,
}

/// Reason why a module was included in the build.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModuleReason {
    /// Module that imported this one
    pub module: Option<String>,

    /// Module identifier that imported this
    pub module_identifier: Option<String>,

    /// Module name that imported this
    pub module_name: Option<String>,

    /// Type of reason (e.g., "harmony import")
    #[serde(rename = "type")]
    pub reason_type: Option<String>,

    /// User request string
    pub user_request: Option<String>,

    /// Location in the importing module
    pub loc: Option<String>,
}

/// Webpack entrypoint configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackEntrypoint {
    /// Entrypoint name
    pub name: Option<String>,

    /// Chunk IDs for this entrypoint
    #[serde(default)]
    pub chunks: Vec<ChunkId>,

    /// Assets for this entrypoint
    #[serde(default)]
    pub assets: Vec<EntrypointAsset>,

    /// Child assets
    #[serde(default)]
    pub children: HashMap<String, Vec<EntrypointAsset>>,

    /// Asset size info
    #[serde(default)]
    pub assets_size: Option<u64>,
}

/// Asset reference in an entrypoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntrypointAsset {
    /// Simple string asset name
    Name(String),
    /// Detailed asset info
    Detailed {
        /// Asset name
        name: String,
        /// Asset size
        #[serde(default)]
        size: u64,
    },
}

/// Named chunk group information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackChunkGroup {
    /// Chunk IDs in this group
    #[serde(default)]
    pub chunks: Vec<ChunkId>,

    /// Assets in this group
    #[serde(default)]
    pub assets: Vec<EntrypointAsset>,

    /// Asset size info
    pub assets_size: Option<u64>,
}

/// Chunk ID can be either a number or string in webpack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ChunkId {
    /// Numeric chunk ID
    Number(u64),
    /// String chunk ID
    String(String),
}

impl Default for ChunkId {
    fn default() -> Self {
        ChunkId::Number(0)
    }
}

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChunkId::Number(n) => write!(f, "{}", n),
            ChunkId::String(s) => write!(f, "{}", s),
        }
    }
}

/// Webpack build error.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackError {
    /// Error message
    pub message: Option<String>,

    /// Module that caused the error
    pub module_identifier: Option<String>,

    /// Module name
    pub module_name: Option<String>,

    /// Location in the module
    pub loc: Option<String>,
}

/// Webpack build warning.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WebpackWarning {
    /// Warning message
    pub message: Option<String>,

    /// Module that caused the warning
    pub module_identifier: Option<String>,

    /// Module name
    pub module_name: Option<String>,

    /// Location in the module
    pub loc: Option<String>,
}

/// Aggregated size information for a single npm package.
#[derive(Debug, Clone, Default)]
pub struct PackageBundleSize {
    /// Package name (e.g., "lodash", "react")
    pub name: String,

    /// Total size in bytes of all modules from this package
    pub total_size: u64,

    /// Number of modules from this package
    pub module_count: usize,

    /// Individual module sizes: (module_path, size)
    pub modules: Vec<(String, u64)>,
}

impl PackageBundleSize {
    /// Create a new PackageBundleSize for a package.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            total_size: 0,
            module_count: 0,
            modules: Vec::new(),
        }
    }

    /// Add a module's size to this package.
    pub fn add_module(&mut self, module_path: String, size: u64) {
        self.total_size += size;
        self.module_count += 1;
        self.modules.push((module_path, size));
    }

    /// Get the percentage of the total bundle this package represents.
    pub fn percentage_of(&self, total_bundle_size: u64) -> f64 {
        if total_bundle_size == 0 {
            0.0
        } else {
            (self.total_size as f64 / total_bundle_size as f64) * 100.0
        }
    }
}

/// Result of parsing and analyzing webpack stats.
#[derive(Debug, Clone, Default)]
pub struct BundleAnalysis {
    /// Total size of all assets
    pub total_asset_size: u64,

    /// Total size of all modules
    pub total_module_size: u64,

    /// Size per npm package
    pub package_sizes: HashMap<String, PackageBundleSize>,

    /// Modules that couldn't be mapped to packages
    pub unmapped_modules: Vec<(String, u64)>,

    /// Number of chunks
    pub chunk_count: usize,

    /// Number of modules
    pub module_count: usize,
}

impl BundleAnalysis {
    /// Get packages sorted by size (largest first).
    pub fn packages_by_size(&self) -> Vec<&PackageBundleSize> {
        let mut packages: Vec<_> = self.package_sizes.values().collect();
        packages.sort_by(|a, b| b.total_size.cmp(&a.total_size));
        packages
    }

    /// Get the size for a specific package.
    pub fn get_package_size(&self, name: &str) -> Option<u64> {
        self.package_sizes.get(name).map(|p| p.total_size)
    }
}

impl WebpackStats {
    /// Parse webpack stats from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the stats.json file
    ///
    /// # Returns
    ///
    /// The parsed `WebpackStats` or an IO error.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use codescope::bundle::webpack::WebpackStats;
    ///
    /// let stats = WebpackStats::from_file("stats.json")?;
    /// println!("Modules: {}", stats.modules.len());
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse webpack stats from a JSON string.
    ///
    /// # Arguments
    ///
    /// * `json` - JSON string containing webpack stats
    ///
    /// # Returns
    ///
    /// The parsed `WebpackStats` or an IO error.
    pub fn parse(json: &str) -> io::Result<Self> {
        serde_json::from_str(json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Analyze the stats and calculate per-package bundle sizes.
    ///
    /// This method:
    /// 1. Iterates through all modules
    /// 2. Extracts the npm package name from each module path
    /// 3. Aggregates sizes per package
    ///
    /// # Returns
    ///
    /// A `BundleAnalysis` containing size information per package.
    pub fn analyze(&self) -> BundleAnalysis {
        let mut analysis = BundleAnalysis {
            chunk_count: self.chunks.len(),
            module_count: self.modules.len(),
            ..Default::default()
        };

        // Calculate total asset size
        analysis.total_asset_size = self.assets.iter().map(|a| a.size).sum();

        // Process all modules (including nested ones)
        self.process_modules(&self.modules, &mut analysis);

        analysis
    }

    /// Process modules recursively (handles concatenated modules).
    fn process_modules(&self, modules: &[WebpackModule], analysis: &mut BundleAnalysis) {
        for module in modules {
            // Get the module path (prefer name, fall back to identifier)
            let module_path = module
                .name
                .as_ref()
                .or(module.identifier.as_ref())
                .cloned()
                .unwrap_or_default();

            // Skip empty paths
            if module_path.is_empty() {
                continue;
            }

            analysis.total_module_size += module.size;

            // Try to extract package name from the module path
            if let Some(package_name) = extract_package_name(&module_path) {
                let package_size = analysis
                    .package_sizes
                    .entry(package_name.clone())
                    .or_insert_with(|| PackageBundleSize::new(package_name));
                package_size.add_module(module_path.clone(), module.size);
            } else {
                // Module doesn't belong to node_modules
                analysis.unmapped_modules.push((module_path, module.size));
            }

            // Process nested modules (concatenated modules)
            if !module.modules.is_empty() {
                self.process_modules(&module.modules, analysis);
            }
        }
    }

    /// Get all modules as a flat list (including nested ones).
    pub fn all_modules(&self) -> Vec<&WebpackModule> {
        let mut result = Vec::new();
        self.collect_modules(&self.modules, &mut result);
        result
    }

    /// Recursively collect all modules.
    fn collect_modules<'a>(
        &'a self,
        modules: &'a [WebpackModule],
        result: &mut Vec<&'a WebpackModule>,
    ) {
        for module in modules {
            result.push(module);
            if !module.modules.is_empty() {
                self.collect_modules(&module.modules, result);
            }
        }
    }
}

/// Extract the npm package name from a webpack module path.
///
/// This handles various path formats:
/// - `./node_modules/lodash/lodash.js` -> `lodash`
/// - `./node_modules/@scope/package/index.js` -> `@scope/package`
/// - `../node_modules/react/cjs/react.production.min.js` -> `react`
/// - `/absolute/path/node_modules/chalk/index.js` -> `chalk`
///
/// # Arguments
///
/// * `module_path` - The full module path from webpack stats
///
/// # Returns
///
/// `Some(package_name)` if the module is from node_modules, `None` otherwise.
///
/// # Example
///
/// ```
/// use codescope::bundle::webpack::extract_package_name;
///
/// assert_eq!(extract_package_name("./node_modules/lodash/lodash.js"), Some("lodash".to_string()));
/// assert_eq!(extract_package_name("./node_modules/@babel/core/lib/index.js"), Some("@babel/core".to_string()));
/// assert_eq!(extract_package_name("./src/app.js"), None);
/// ```
pub fn extract_package_name(module_path: &str) -> Option<String> {
    // Find the node_modules segment in the path
    let node_modules_marker = "node_modules/";

    // Find the last occurrence of node_modules (handles nested node_modules)
    let nm_pos = module_path.rfind(node_modules_marker)?;
    let after_nm = &module_path[nm_pos + node_modules_marker.len()..];

    // Split by '/' to get path segments
    let segments: Vec<&str> = after_nm.split('/').collect();

    if segments.is_empty() {
        return None;
    }

    // Check if it's a scoped package (@org/package)
    if segments[0].starts_with('@') {
        // Scoped package: need @scope/package
        if segments.len() >= 2 {
            Some(format!("{}/{}", segments[0], segments[1]))
        } else {
            None
        }
    } else {
        // Regular package: just the first segment
        Some(segments[0].to_string())
    }
}

/// Format a byte size as a human-readable string.
///
/// # Example
///
/// ```
/// use codescope::bundle::webpack::format_size;
///
/// assert_eq!(format_size(1024), "1.00 KB");
/// assert_eq!(format_size(1048576), "1.00 MB");
/// ```
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name_regular() {
        assert_eq!(
            extract_package_name("./node_modules/lodash/lodash.js"),
            Some("lodash".to_string())
        );
        assert_eq!(
            extract_package_name("./node_modules/react/index.js"),
            Some("react".to_string())
        );
        assert_eq!(
            extract_package_name("./node_modules/chalk/source/index.js"),
            Some("chalk".to_string())
        );
    }

    #[test]
    fn test_extract_package_name_scoped() {
        assert_eq!(
            extract_package_name("./node_modules/@babel/core/lib/index.js"),
            Some("@babel/core".to_string())
        );
        assert_eq!(
            extract_package_name("./node_modules/@types/react/index.d.ts"),
            Some("@types/react".to_string())
        );
        assert_eq!(
            extract_package_name("./node_modules/@scope/pkg/dist/main.js"),
            Some("@scope/pkg".to_string())
        );
    }

    #[test]
    fn test_extract_package_name_nested_node_modules() {
        // When there are nested node_modules, use the innermost one
        assert_eq!(
            extract_package_name("./node_modules/pkg-a/node_modules/pkg-b/index.js"),
            Some("pkg-b".to_string())
        );
    }

    #[test]
    fn test_extract_package_name_absolute_path() {
        assert_eq!(
            extract_package_name("/home/user/project/node_modules/react/index.js"),
            Some("react".to_string())
        );
    }

    #[test]
    fn test_extract_package_name_no_node_modules() {
        assert_eq!(extract_package_name("./src/app.js"), None);
        assert_eq!(extract_package_name("./components/Button.tsx"), None);
        assert_eq!(extract_package_name("webpack/runtime/define"), None);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_package_bundle_size() {
        let mut pkg = PackageBundleSize::new("lodash");
        assert_eq!(pkg.name, "lodash");
        assert_eq!(pkg.total_size, 0);
        assert_eq!(pkg.module_count, 0);

        pkg.add_module("lodash/lodash.js".to_string(), 1000);
        pkg.add_module("lodash/fp.js".to_string(), 500);

        assert_eq!(pkg.total_size, 1500);
        assert_eq!(pkg.module_count, 2);
        assert_eq!(pkg.modules.len(), 2);
    }

    #[test]
    fn test_package_bundle_size_percentage() {
        let mut pkg = PackageBundleSize::new("test");
        pkg.total_size = 250;

        assert!((pkg.percentage_of(1000) - 25.0).abs() < 0.01);
        assert_eq!(pkg.percentage_of(0), 0.0);
    }

    #[test]
    fn test_chunk_id_display() {
        let num_id = ChunkId::Number(42);
        let str_id = ChunkId::String("main".to_string());

        assert_eq!(format!("{}", num_id), "42");
        assert_eq!(format!("{}", str_id), "main");
    }

    #[test]
    fn test_parse_minimal_stats() {
        let json = r#"{
            "version": "5.88.0",
            "hash": "abc123",
            "time": 1234,
            "modules": [
                {
                    "name": "./node_modules/react/index.js",
                    "size": 1000
                },
                {
                    "name": "./node_modules/lodash/lodash.js",
                    "size": 2000
                },
                {
                    "name": "./src/app.js",
                    "size": 500
                }
            ],
            "assets": [
                {
                    "name": "main.js",
                    "size": 3500
                }
            ],
            "chunks": []
        }"#;

        let stats = WebpackStats::parse(json).unwrap();
        assert_eq!(stats.version, Some("5.88.0".to_string()));
        assert_eq!(stats.modules.len(), 3);
        assert_eq!(stats.assets.len(), 1);

        let analysis = stats.analyze();
        assert_eq!(analysis.package_sizes.len(), 2); // react and lodash
        assert_eq!(analysis.unmapped_modules.len(), 1); // src/app.js

        assert_eq!(
            analysis.package_sizes.get("react").unwrap().total_size,
            1000
        );
        assert_eq!(
            analysis.package_sizes.get("lodash").unwrap().total_size,
            2000
        );
    }

    #[test]
    fn test_parse_nested_modules() {
        let json = r#"{
            "modules": [
                {
                    "name": "./node_modules/pkg/index.js",
                    "size": 100,
                    "modules": [
                        {
                            "name": "./node_modules/pkg/util.js",
                            "size": 50
                        }
                    ]
                }
            ],
            "assets": [],
            "chunks": []
        }"#;

        let stats = WebpackStats::parse(json).unwrap();
        let analysis = stats.analyze();

        // Both the parent and nested module should be counted
        let pkg_size = analysis.package_sizes.get("pkg").unwrap();
        assert_eq!(pkg_size.total_size, 150);
        assert_eq!(pkg_size.module_count, 2);
    }

    #[test]
    fn test_bundle_analysis_packages_by_size() {
        let json = r#"{
            "modules": [
                { "name": "./node_modules/large/index.js", "size": 5000 },
                { "name": "./node_modules/medium/index.js", "size": 2000 },
                { "name": "./node_modules/small/index.js", "size": 500 }
            ],
            "assets": [],
            "chunks": []
        }"#;

        let stats = WebpackStats::parse(json).unwrap();
        let analysis = stats.analyze();

        let sorted = analysis.packages_by_size();
        assert_eq!(sorted[0].name, "large");
        assert_eq!(sorted[1].name, "medium");
        assert_eq!(sorted[2].name, "small");
    }

    #[test]
    fn test_all_modules_flattens_nested() {
        let json = r#"{
            "modules": [
                {
                    "name": "parent",
                    "size": 100,
                    "modules": [
                        { "name": "child1", "size": 50 },
                        { "name": "child2", "size": 50 }
                    ]
                }
            ],
            "assets": [],
            "chunks": []
        }"#;

        let stats = WebpackStats::parse(json).unwrap();
        let all_modules = stats.all_modules();

        assert_eq!(all_modules.len(), 3);
    }

    #[test]
    fn test_parse_empty_stats() {
        let json = "{}";
        let stats = WebpackStats::parse(json).unwrap();

        assert!(stats.modules.is_empty());
        assert!(stats.assets.is_empty());
        assert!(stats.chunks.is_empty());
    }

    #[test]
    fn test_scoped_packages_analyzed_correctly() {
        let json = r#"{
            "modules": [
                { "name": "./node_modules/@babel/core/lib/index.js", "size": 1000 },
                { "name": "./node_modules/@babel/core/lib/parse.js", "size": 500 },
                { "name": "./node_modules/@babel/preset-env/lib/index.js", "size": 2000 }
            ],
            "assets": [],
            "chunks": []
        }"#;

        let stats = WebpackStats::parse(json).unwrap();
        let analysis = stats.analyze();

        assert_eq!(analysis.package_sizes.len(), 2);
        assert_eq!(
            analysis
                .package_sizes
                .get("@babel/core")
                .unwrap()
                .total_size,
            1500
        );
        assert_eq!(
            analysis
                .package_sizes
                .get("@babel/preset-env")
                .unwrap()
                .total_size,
            2000
        );
    }
}
