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

pub mod savings;
pub mod webpack;

// Re-export main types for convenience
pub use savings::{
    PackageSavings, SavingsCalculator, SavingsCategory, SavingsReport, SavingsSummary,
};
pub use webpack::{
    extract_package_name, format_size, BundleAnalysis, PackageBundleSize, WebpackAsset,
    WebpackChunk, WebpackModule, WebpackStats,
};

use crate::graph::DependencyGraph;
use crate::ui::tree::TreeNode;
use std::collections::HashMap;

/// Maps bundle analysis results to a format suitable for applying to a DependencyGraph.
///
/// This function extracts package sizes from a BundleAnalysis and returns a HashMap
/// that can be used with `DependencyGraph::apply_bundle_sizes()`.
///
/// # Arguments
///
/// * `analysis` - The bundle analysis containing per-package size information
///
/// # Returns
///
/// A HashMap mapping package names to (total_size, module_count) tuples.
///
/// # Example
///
/// ```ignore
/// use codescope::bundle::{WebpackStats, bundle_sizes_to_map};
///
/// let stats = WebpackStats::from_file("stats.json")?;
/// let analysis = stats.analyze();
/// let sizes = bundle_sizes_to_map(&analysis);
///
/// // Use with dependency graph
/// graph.apply_bundle_sizes(&sizes);
/// ```
pub fn bundle_sizes_to_map(analysis: &BundleAnalysis) -> HashMap<String, (u64, usize)> {
    analysis
        .package_sizes
        .iter()
        .map(|(name, pkg_size)| (name.clone(), (pkg_size.total_size, pkg_size.module_count)))
        .collect()
}

/// Applies bundle size information from a BundleAnalysis to a DependencyGraph.
///
/// This is a convenience function that extracts package sizes from the analysis
/// and applies them to matching nodes in the graph.
///
/// # Arguments
///
/// * `graph` - The dependency graph to update
/// * `analysis` - The bundle analysis containing per-package size information
///
/// # Returns
///
/// The number of nodes that were updated with size information.
///
/// # Example
///
/// ```ignore
/// use codescope::bundle::{WebpackStats, apply_bundle_sizes_to_graph};
/// use codescope::graph::DependencyGraph;
///
/// let stats = WebpackStats::from_file("stats.json")?;
/// let analysis = stats.analyze();
///
/// let mut graph = DependencyGraph::new();
/// // ... populate graph ...
///
/// let updated = apply_bundle_sizes_to_graph(&mut graph, &analysis);
/// println!("Updated {} nodes with bundle sizes", updated);
/// ```
pub fn apply_bundle_sizes_to_graph(graph: &mut DependencyGraph, analysis: &BundleAnalysis) -> usize {
    let sizes = bundle_sizes_to_map(analysis);
    graph.apply_bundle_sizes(&sizes)
}

/// Applies bundle size information from a BundleAnalysis to a TreeNode.
///
/// This recursively updates the tree and all children with bundle size information.
///
/// # Arguments
///
/// * `tree` - The tree node to update
/// * `analysis` - The bundle analysis containing per-package size information
///
/// # Example
///
/// ```ignore
/// use codescope::bundle::{WebpackStats, apply_bundle_sizes_to_tree};
/// use codescope::ui::tree::TreeNode;
///
/// let stats = WebpackStats::from_file("stats.json")?;
/// let analysis = stats.analyze();
///
/// let mut tree = TreeNode::new("my-app".to_string(), "1.0.0".to_string());
/// // ... build tree ...
///
/// apply_bundle_sizes_to_tree(&mut tree, &analysis);
/// ```
pub fn apply_bundle_sizes_to_tree(tree: &mut TreeNode, analysis: &BundleAnalysis) {
    let sizes = bundle_sizes_to_map(analysis);
    tree.apply_bundle_sizes(&sizes);
}

/// Result of matching bundle modules to dependencies.
///
/// Contains statistics about the matching process.
#[derive(Debug, Clone, Default)]
pub struct MatchResult {
    /// Number of dependencies that were matched to bundle data
    pub matched_count: usize,
    /// Number of dependencies that had no bundle data
    pub unmatched_count: usize,
    /// Total bundle size of matched dependencies
    pub matched_size: u64,
    /// Package names that were in the bundle but not in dependencies
    pub extra_packages: Vec<String>,
    /// Package names that were in dependencies but not in the bundle
    pub missing_packages: Vec<String>,
}

impl MatchResult {
    /// Returns true if all dependencies were matched
    pub fn is_complete(&self) -> bool {
        self.unmatched_count == 0 && self.missing_packages.is_empty()
    }

    /// Returns the percentage of dependencies that were matched
    pub fn match_percentage(&self) -> f64 {
        let total = self.matched_count + self.unmatched_count;
        if total == 0 {
            100.0
        } else {
            (self.matched_count as f64 / total as f64) * 100.0
        }
    }
}

/// Matches bundle analysis data to dependencies and returns statistics.
///
/// This function compares the packages found in the bundle analysis with
/// the nodes in the dependency graph to determine match coverage.
///
/// # Arguments
///
/// * `graph` - The dependency graph to match against
/// * `analysis` - The bundle analysis containing per-package size information
///
/// # Returns
///
/// A `MatchResult` containing matching statistics.
///
/// # Example
///
/// ```ignore
/// use codescope::bundle::{WebpackStats, match_bundle_to_dependencies};
///
/// let stats = WebpackStats::from_file("stats.json")?;
/// let analysis = stats.analyze();
///
/// let result = match_bundle_to_dependencies(&graph, &analysis);
/// println!("Matched {}% of dependencies", result.match_percentage());
/// ```
pub fn match_bundle_to_dependencies(
    graph: &DependencyGraph,
    analysis: &BundleAnalysis,
) -> MatchResult {
    let mut result = MatchResult::default();

    // Get all package names from the graph
    let graph_packages: std::collections::HashSet<&str> = graph
        .get_all_nodes()
        .iter()
        .map(|n| n.name.as_str())
        .collect();

    // Get all package names from the bundle analysis
    let bundle_packages: std::collections::HashSet<&str> =
        analysis.package_sizes.keys().map(|s| s.as_str()).collect();

    // Find matched packages
    for &pkg_name in &graph_packages {
        if let Some(pkg_size) = analysis.package_sizes.get(pkg_name) {
            result.matched_count += 1;
            result.matched_size += pkg_size.total_size;
        } else {
            result.unmatched_count += 1;
            result.missing_packages.push(pkg_name.to_string());
        }
    }

    // Find extra packages (in bundle but not in dependencies)
    for &pkg_name in &bundle_packages {
        if !graph_packages.contains(pkg_name) {
            result.extra_packages.push(pkg_name.to_string());
        }
    }

    result
}

/// Calculates the transitive bundle size for each dependency.
///
/// This function computes the total bundle size contribution of each package
/// including its transitive dependencies. This helps identify packages that
/// might be small themselves but bring in many large dependencies.
///
/// # Arguments
///
/// * `graph` - The dependency graph with bundle sizes already applied
///
/// # Returns
///
/// A HashMap mapping package names to their transitive bundle size.
///
/// # Note
///
/// This is an approximation since the same transitive dependency might be
/// shared by multiple packages. The size is attributed to each package that
/// depends on it.
pub fn calculate_transitive_sizes(graph: &DependencyGraph) -> HashMap<String, u64> {
    let mut transitive_sizes: HashMap<String, u64> = HashMap::new();

    for node in graph.get_all_nodes() {
        let own_size = node.bundle_size.unwrap_or(0);
        let transitive_size = calculate_transitive_size_for_node(graph, &node.name, own_size);
        transitive_sizes.insert(node.name.clone(), transitive_size);
    }

    transitive_sizes
}

/// Helper function to recursively calculate transitive size for a node.
fn calculate_transitive_size_for_node(
    graph: &DependencyGraph,
    package_name: &str,
    own_size: u64,
) -> u64 {
    let mut total = own_size;
    let mut visited = std::collections::HashSet::new();
    visited.insert(package_name.to_string());

    let mut stack = vec![package_name.to_string()];

    while let Some(current) = stack.pop() {
        for dep in graph.get_dependencies(&current) {
            if !visited.contains(&dep.name) {
                visited.insert(dep.name.clone());
                total += dep.bundle_size.unwrap_or(0);
                stack.push(dep.name.clone());
            }
        }
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DependencyType;

    #[test]
    fn test_bundle_sizes_to_map() {
        let mut analysis = BundleAnalysis::default();
        let mut pkg = PackageBundleSize::new("react");
        pkg.add_module("react/index.js".to_string(), 1000);
        pkg.add_module("react/cjs.js".to_string(), 500);
        analysis.package_sizes.insert("react".to_string(), pkg);

        let mut pkg2 = PackageBundleSize::new("lodash");
        pkg2.add_module("lodash/index.js".to_string(), 2000);
        analysis.package_sizes.insert("lodash".to_string(), pkg2);

        let map = bundle_sizes_to_map(&analysis);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get("react"), Some(&(1500, 2)));
        assert_eq!(map.get("lodash"), Some(&(2000, 1)));
    }

    #[test]
    fn test_apply_bundle_sizes_to_graph() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.0.0", DependencyType::Production);
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.add_dependency("typescript", "5.0.0", DependencyType::Development);

        let mut analysis = BundleAnalysis::default();
        let mut pkg = PackageBundleSize::new("react");
        pkg.add_module("react/index.js".to_string(), 1000);
        analysis.package_sizes.insert("react".to_string(), pkg);

        let mut pkg2 = PackageBundleSize::new("lodash");
        pkg2.add_module("lodash/index.js".to_string(), 2000);
        analysis.package_sizes.insert("lodash".to_string(), pkg2);

        let updated = apply_bundle_sizes_to_graph(&mut graph, &analysis);

        assert_eq!(updated, 2);
        assert_eq!(graph.get_node("react").unwrap().bundle_size, Some(1000));
        assert_eq!(graph.get_node("lodash").unwrap().bundle_size, Some(2000));
        assert_eq!(graph.get_node("typescript").unwrap().bundle_size, None);
    }

    #[test]
    fn test_apply_bundle_sizes_to_tree() {
        let mut root = TreeNode::new("my-app".to_string(), "1.0.0".to_string());
        let react = TreeNode::new("react".to_string(), "18.0.0".to_string());
        let lodash = TreeNode::new("lodash".to_string(), "4.17.0".to_string());
        root.add_child(react);
        root.add_child(lodash);

        let mut analysis = BundleAnalysis::default();
        let mut pkg = PackageBundleSize::new("react");
        pkg.add_module("react/index.js".to_string(), 1500);
        analysis.package_sizes.insert("react".to_string(), pkg);

        apply_bundle_sizes_to_tree(&mut root, &analysis);

        assert_eq!(root.bundle_size, None);
        assert_eq!(root.children[0].bundle_size, Some(1500));
        assert_eq!(root.children[0].module_count, Some(1));
        assert_eq!(root.children[1].bundle_size, None);
    }

    #[test]
    fn test_match_bundle_to_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.0.0", DependencyType::Production);
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.add_dependency("typescript", "5.0.0", DependencyType::Development);

        let mut analysis = BundleAnalysis::default();
        let mut pkg = PackageBundleSize::new("react");
        pkg.add_module("react/index.js".to_string(), 1000);
        analysis.package_sizes.insert("react".to_string(), pkg);

        let mut pkg2 = PackageBundleSize::new("chalk");
        pkg2.add_module("chalk/index.js".to_string(), 500);
        analysis.package_sizes.insert("chalk".to_string(), pkg2);

        let result = match_bundle_to_dependencies(&graph, &analysis);

        assert_eq!(result.matched_count, 1); // react
        assert_eq!(result.unmatched_count, 2); // lodash, typescript
        assert_eq!(result.matched_size, 1000);
        assert!(result.extra_packages.contains(&"chalk".to_string()));
        assert!(result.missing_packages.contains(&"lodash".to_string()));
        assert!(result.missing_packages.contains(&"typescript".to_string()));
    }

    #[test]
    fn test_match_result_percentage() {
        let mut result = MatchResult::default();
        result.matched_count = 3;
        result.unmatched_count = 1;

        assert!((result.match_percentage() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_match_result_complete() {
        let mut result = MatchResult::default();
        result.matched_count = 5;
        result.unmatched_count = 0;
        result.missing_packages = vec![];

        assert!(result.is_complete());

        result.missing_packages.push("missing".to_string());
        assert!(!result.is_complete());
    }

    #[test]
    fn test_calculate_transitive_sizes() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("app", "1.0.0", DependencyType::Production);
        graph.add_dependency("react", "18.0.0", DependencyType::Production);
        graph.add_dependency("scheduler", "0.23.0", DependencyType::Production);

        graph.add_edge("app", "react");
        graph.add_edge("react", "scheduler");

        // Apply bundle sizes
        let mut sizes = HashMap::new();
        sizes.insert("app".to_string(), (100_u64, 1_usize));
        sizes.insert("react".to_string(), (1000_u64, 5_usize));
        sizes.insert("scheduler".to_string(), (500_u64, 2_usize));
        graph.apply_bundle_sizes(&sizes);

        let transitive = calculate_transitive_sizes(&graph);

        // app has its own size + react + scheduler
        assert_eq!(transitive.get("app"), Some(&1600));
        // react has its own size + scheduler
        assert_eq!(transitive.get("react"), Some(&1500));
        // scheduler has only its own size
        assert_eq!(transitive.get("scheduler"), Some(&500));
    }

    #[test]
    fn test_calculate_transitive_sizes_with_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", DependencyType::Production);
        graph.add_dependency("b", "1.0.0", DependencyType::Production);
        graph.add_dependency("c", "1.0.0", DependencyType::Production);

        graph.add_edge("a", "b");
        graph.add_edge("b", "c");
        graph.add_edge("c", "a"); // cycle

        let mut sizes = HashMap::new();
        sizes.insert("a".to_string(), (100_u64, 1_usize));
        sizes.insert("b".to_string(), (200_u64, 1_usize));
        sizes.insert("c".to_string(), (300_u64, 1_usize));
        graph.apply_bundle_sizes(&sizes);

        let transitive = calculate_transitive_sizes(&graph);

        // Each node should include all nodes in the cycle
        assert_eq!(transitive.get("a"), Some(&600));
        assert_eq!(transitive.get("b"), Some(&600));
        assert_eq!(transitive.get("c"), Some(&600));
    }
}
