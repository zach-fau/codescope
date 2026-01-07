//! Dependency graph implementation using petgraph.
//!
//! Provides a directed graph structure for modeling package dependencies,
//! with support for different dependency types, cycle detection, and traversal.

use petgraph::algo::is_cyclic_directed;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::{HashMap, HashSet};

/// Represents the type of dependency relationship.
///
/// Different dependency types have different implications for bundling
/// and runtime behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DependencyType {
    /// Production dependencies - required at runtime
    #[default]
    Production,
    /// Development dependencies - only needed during development
    Development,
    /// Peer dependencies - expected to be provided by the consumer
    Peer,
    /// Optional dependencies - may or may not be installed
    Optional,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Production => write!(f, "production"),
            Self::Development => write!(f, "dev"),
            Self::Peer => write!(f, "peer"),
            Self::Optional => write!(f, "optional"),
        }
    }
}

/// Represents a node in the dependency graph.
///
/// Each node contains metadata about a single package dependency.
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Package name (e.g., "react", "lodash")
    pub name: String,
    /// Version specification (e.g., "^18.2.0", "1.0.0")
    pub version: String,
    /// Type of dependency relationship
    pub dep_type: DependencyType,
    /// Distance from root package (0 = direct dependency)
    pub depth: usize,
    /// Bundle size in bytes (from webpack/bundler stats)
    pub bundle_size: Option<u64>,
    /// Number of modules from this package included in the bundle
    pub module_count: Option<usize>,
}

impl DependencyNode {
    /// Creates a new dependency node.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `version` - Version specification
    /// * `dep_type` - Type of dependency
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyNode, DependencyType};
    ///
    /// let node = DependencyNode::new("react", "18.2.0", DependencyType::Production);
    /// assert_eq!(node.name, "react");
    /// assert_eq!(node.depth, 0);
    /// ```
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        dep_type: DependencyType,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            dep_type,
            depth: 0,
            bundle_size: None,
            module_count: None,
        }
    }

    /// Creates a new node with a specified depth.
    pub fn with_depth(
        name: impl Into<String>,
        version: impl Into<String>,
        dep_type: DependencyType,
        depth: usize,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            dep_type,
            depth,
            bundle_size: None,
            module_count: None,
        }
    }

    /// Creates a new node with bundle size information.
    pub fn with_bundle_size(
        name: impl Into<String>,
        version: impl Into<String>,
        dep_type: DependencyType,
        bundle_size: u64,
        module_count: usize,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            dep_type,
            depth: 0,
            bundle_size: Some(bundle_size),
            module_count: Some(module_count),
        }
    }

    /// Sets the bundle size for this node.
    pub fn set_bundle_size(&mut self, size: u64, module_count: usize) {
        self.bundle_size = Some(size);
        self.module_count = Some(module_count);
    }

    /// Returns true if this node has bundle size information.
    pub fn has_bundle_size(&self) -> bool {
        self.bundle_size.is_some()
    }
}

/// Represents an edge in the dependency graph.
///
/// Edges connect a dependent package to its dependency, with optional
/// metadata about the relationship.
#[derive(Debug, Clone, Default)]
pub struct DependencyEdge {
    /// Whether this dependency is optional
    pub is_optional: bool,
}

impl DependencyEdge {
    /// Creates a new required (non-optional) dependency edge.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new optional dependency edge.
    pub fn optional() -> Self {
        Self { is_optional: true }
    }
}

/// A directed graph representing package dependencies.
///
/// The graph uses petgraph's `DiGraph` internally, with nodes representing
/// packages and edges representing dependency relationships. Edges point
/// from the dependent package to its dependency.
///
/// # Example
///
/// ```rust
/// use codescope::graph::{DependencyGraph, DependencyType};
///
/// let mut graph = DependencyGraph::new();
///
/// // Add packages
/// graph.add_dependency("my-app", "1.0.0", DependencyType::Production);
/// graph.add_dependency("react", "18.2.0", DependencyType::Production);
/// graph.add_dependency("react-dom", "18.2.0", DependencyType::Production);
///
/// // Add relationships (my-app depends on react and react-dom)
/// graph.add_edge("my-app", "react");
/// graph.add_edge("my-app", "react-dom");
/// graph.add_edge("react-dom", "react"); // react-dom depends on react
///
/// assert_eq!(graph.node_count(), 3);
/// assert_eq!(graph.edge_count(), 3);
/// ```
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// The underlying directed graph
    graph: DiGraph<DependencyNode, DependencyEdge>,
    /// Maps package names to their node indices for O(1) lookup
    node_indices: HashMap<String, NodeIndex>,
    /// Tracks version requirements for each package: package_name -> [(version, required_by)]
    version_requirements: HashMap<String, Vec<VersionRequirement>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// Creates a new empty dependency graph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::DependencyGraph;
    ///
    /// let graph = DependencyGraph::new();
    /// assert_eq!(graph.node_count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            version_requirements: HashMap::new(),
        }
    }

    /// Creates a new graph with pre-allocated capacity.
    ///
    /// Use this when you know approximately how many nodes and edges
    /// will be added to avoid reallocations.
    ///
    /// # Arguments
    ///
    /// * `nodes` - Expected number of nodes
    /// * `edges` - Expected number of edges
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self {
            graph: DiGraph::with_capacity(nodes, edges),
            node_indices: HashMap::with_capacity(nodes),
            version_requirements: HashMap::with_capacity(nodes),
        }
    }

    /// Adds a dependency to the graph.
    ///
    /// If a dependency with the same name already exists, returns its
    /// existing node index without modification.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `version` - Version specification
    /// * `dep_type` - Type of dependency
    ///
    /// # Returns
    ///
    /// The `NodeIndex` of the added or existing node.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// let idx = graph.add_dependency("react", "18.2.0", DependencyType::Production);
    /// assert!(graph.get_node("react").is_some());
    /// ```
    pub fn add_dependency(
        &mut self,
        name: &str,
        version: &str,
        dep_type: DependencyType,
    ) -> NodeIndex {
        // Return existing index if node already exists
        if let Some(&idx) = self.node_indices.get(name) {
            return idx;
        }

        // Create and add new node
        let node = DependencyNode::new(name, version, dep_type);
        let idx = self.graph.add_node(node);
        self.node_indices.insert(name.to_string(), idx);
        idx
    }

    /// Adds a dependency with a specific depth.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name
    /// * `version` - Version specification
    /// * `dep_type` - Type of dependency
    /// * `depth` - Distance from root package
    ///
    /// # Returns
    ///
    /// The `NodeIndex` of the added or existing node.
    pub fn add_dependency_with_depth(
        &mut self,
        name: &str,
        version: &str,
        dep_type: DependencyType,
        depth: usize,
    ) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(name) {
            return idx;
        }

        let node = DependencyNode::with_depth(name, version, dep_type, depth);
        let idx = self.graph.add_node(node);
        self.node_indices.insert(name.to_string(), idx);
        idx
    }

    /// Adds an edge between two dependencies.
    ///
    /// Creates an edge from `from` (the dependent) to `to` (the dependency).
    /// Both nodes must already exist in the graph.
    ///
    /// # Arguments
    ///
    /// * `from` - Name of the dependent package
    /// * `to` - Name of the dependency
    ///
    /// # Returns
    ///
    /// `true` if the edge was added, `false` if either node doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("react-dom", "18.2.0", DependencyType::Production);
    /// graph.add_dependency("react", "18.2.0", DependencyType::Production);
    ///
    /// assert!(graph.add_edge("react-dom", "react"));
    /// assert!(!graph.add_edge("nonexistent", "react")); // Returns false
    /// ```
    pub fn add_edge(&mut self, from: &str, to: &str) -> bool {
        self.add_edge_with_metadata(from, to, DependencyEdge::new())
    }

    /// Adds an optional dependency edge.
    ///
    /// # Arguments
    ///
    /// * `from` - Name of the dependent package
    /// * `to` - Name of the optional dependency
    ///
    /// # Returns
    ///
    /// `true` if the edge was added, `false` if either node doesn't exist.
    pub fn add_optional_edge(&mut self, from: &str, to: &str) -> bool {
        self.add_edge_with_metadata(from, to, DependencyEdge::optional())
    }

    /// Adds an edge with custom metadata.
    ///
    /// # Arguments
    ///
    /// * `from` - Name of the dependent package
    /// * `to` - Name of the dependency
    /// * `edge` - Edge metadata
    ///
    /// # Returns
    ///
    /// `true` if the edge was added, `false` if either node doesn't exist.
    pub fn add_edge_with_metadata(&mut self, from: &str, to: &str, edge: DependencyEdge) -> bool {
        let from_idx = match self.node_indices.get(from) {
            Some(&idx) => idx,
            None => return false,
        };
        let to_idx = match self.node_indices.get(to) {
            Some(&idx) => idx,
            None => return false,
        };

        self.graph.add_edge(from_idx, to_idx, edge);
        true
    }

    /// Gets a reference to a dependency node by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name to look up
    ///
    /// # Returns
    ///
    /// `Some(&DependencyNode)` if found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("react", "18.2.0", DependencyType::Production);
    ///
    /// if let Some(node) = graph.get_node("react") {
    ///     assert_eq!(node.version, "18.2.0");
    /// }
    /// ```
    pub fn get_node(&self, name: &str) -> Option<&DependencyNode> {
        self.node_indices
            .get(name)
            .and_then(|&idx| self.graph.node_weight(idx))
    }

    /// Gets the dependencies of a package (outgoing edges).
    ///
    /// Returns packages that the specified package depends on.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name to get dependencies for
    ///
    /// # Returns
    ///
    /// A vector of references to dependency nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("my-app", "1.0.0", DependencyType::Production);
    /// graph.add_dependency("react", "18.2.0", DependencyType::Production);
    /// graph.add_edge("my-app", "react");
    ///
    /// let deps = graph.get_dependencies("my-app");
    /// assert_eq!(deps.len(), 1);
    /// assert_eq!(deps[0].name, "react");
    /// ```
    pub fn get_dependencies(&self, name: &str) -> Vec<&DependencyNode> {
        let Some(&idx) = self.node_indices.get(name) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(idx, Direction::Outgoing)
            .filter_map(|edge| self.graph.node_weight(edge.target()))
            .collect()
    }

    /// Gets the dependents of a package (incoming edges).
    ///
    /// Returns packages that depend on the specified package.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name to get dependents for
    ///
    /// # Returns
    ///
    /// A vector of references to dependent nodes.
    pub fn get_dependents(&self, name: &str) -> Vec<&DependencyNode> {
        let Some(&idx) = self.node_indices.get(name) else {
            return Vec::new();
        };

        self.graph
            .edges_directed(idx, Direction::Incoming)
            .filter_map(|edge| self.graph.node_weight(edge.source()))
            .collect()
    }

    /// Gets all nodes in the graph.
    ///
    /// # Returns
    ///
    /// A vector of references to all dependency nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("react", "18.2.0", DependencyType::Production);
    /// graph.add_dependency("lodash", "4.17.21", DependencyType::Production);
    ///
    /// assert_eq!(graph.get_all_nodes().len(), 2);
    /// ```
    pub fn get_all_nodes(&self) -> Vec<&DependencyNode> {
        self.graph.node_weights().collect()
    }

    /// Checks if the graph contains cycles.
    ///
    /// Circular dependencies can cause issues in bundling and runtime.
    ///
    /// # Returns
    ///
    /// `true` if the graph contains at least one cycle.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("a", "1.0.0", DependencyType::Production);
    /// graph.add_dependency("b", "1.0.0", DependencyType::Production);
    /// graph.add_edge("a", "b");
    /// graph.add_edge("b", "a"); // Creates a cycle
    ///
    /// assert!(graph.has_cycles());
    /// ```
    pub fn has_cycles(&self) -> bool {
        is_cyclic_directed(&self.graph)
    }

    /// Detects and returns all cycles in the graph.
    ///
    /// Uses depth-first search to find strongly connected components
    /// that form cycles.
    ///
    /// # Returns
    ///
    /// A vector of cycles, where each cycle is a vector of package names.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("a", "1.0.0", DependencyType::Production);
    /// graph.add_dependency("b", "1.0.0", DependencyType::Production);
    /// graph.add_dependency("c", "1.0.0", DependencyType::Production);
    /// graph.add_edge("a", "b");
    /// graph.add_edge("b", "c");
    /// graph.add_edge("c", "a"); // Creates cycle: a -> b -> c -> a
    ///
    /// let cycles = graph.detect_cycles();
    /// assert!(!cycles.is_empty());
    /// ```
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        use petgraph::algo::tarjan_scc;

        let sccs = tarjan_scc(&self.graph);
        let mut cycles = Vec::new();

        for scc in sccs {
            // A strongly connected component is a cycle if it has more than one node,
            // or if it's a single node with a self-loop
            if scc.len() > 1 {
                let cycle: Vec<String> = scc
                    .iter()
                    .filter_map(|&idx| self.graph.node_weight(idx))
                    .map(|node| node.name.clone())
                    .collect();
                cycles.push(cycle);
            } else if scc.len() == 1 {
                // Check for self-loop
                let idx = scc[0];
                if self.graph.contains_edge(idx, idx) {
                    if let Some(node) = self.graph.node_weight(idx) {
                        cycles.push(vec![node.name.clone()]);
                    }
                }
            }
        }

        cycles
    }

    /// Returns a set of package names that are part of any cycle.
    ///
    /// This is useful for marking nodes in the UI that participate in
    /// circular dependencies.
    ///
    /// # Returns
    ///
    /// A `HashSet` of package names that are part of at least one cycle.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("a", "1.0.0", DependencyType::Production);
    /// graph.add_dependency("b", "1.0.0", DependencyType::Production);
    /// graph.add_dependency("c", "1.0.0", DependencyType::Production);
    /// graph.add_dependency("d", "1.0.0", DependencyType::Production);
    /// graph.add_edge("a", "b");
    /// graph.add_edge("b", "c");
    /// graph.add_edge("c", "a"); // Creates cycle: a -> b -> c -> a
    /// graph.add_edge("a", "d"); // d is not part of the cycle
    ///
    /// let cycle_nodes = graph.get_nodes_in_cycles();
    /// assert!(cycle_nodes.contains("a"));
    /// assert!(cycle_nodes.contains("b"));
    /// assert!(cycle_nodes.contains("c"));
    /// assert!(!cycle_nodes.contains("d"));
    /// ```
    pub fn get_nodes_in_cycles(&self) -> HashSet<String> {
        let cycles = self.detect_cycles();
        let mut cycle_nodes = HashSet::new();

        for cycle in cycles {
            for node_name in cycle {
                cycle_nodes.insert(node_name);
            }
        }

        cycle_nodes
    }

    /// Returns detailed cycle information including the cycle path.
    ///
    /// For each cycle detected, returns the list of package names in the order
    /// they form the cycle (note: the last element connects back to the first).
    ///
    /// # Returns
    ///
    /// A vector of `CycleInfo` structs containing cycle details.
    pub fn get_cycle_details(&self) -> Vec<CycleInfo> {
        self.detect_cycles()
            .into_iter()
            .map(|nodes| CycleInfo { nodes })
            .collect()
    }

    /// Tracks a version requirement for a package.
    ///
    /// Records that `required_by` package requires `package_name` at `version`.
    /// This information is used to detect version conflicts.
    ///
    /// # Arguments
    ///
    /// * `package_name` - The dependency package name
    /// * `version` - The version specification
    /// * `required_by` - The package that requires this dependency
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("lodash", "^4.17.0", DependencyType::Production);
    /// graph.track_version_requirement("lodash", "^4.17.0", "my-app");
    /// graph.track_version_requirement("lodash", "^4.16.0", "other-pkg");
    /// ```
    pub fn track_version_requirement(
        &mut self,
        package_name: &str,
        version: &str,
        required_by: &str,
    ) {
        let requirements = self
            .version_requirements
            .entry(package_name.to_string())
            .or_default();
        requirements.push(VersionRequirement::new(version, required_by));
    }

    /// Detects version conflicts in the dependency graph.
    ///
    /// A conflict exists when the same package is required at different
    /// versions by different dependents.
    ///
    /// # Returns
    ///
    /// A vector of `VersionConflict` structs containing conflict details.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
    /// graph.track_version_requirement("lodash", "^4.17.0", "my-app");
    /// graph.track_version_requirement("lodash", "^4.16.0", "other-pkg");
    ///
    /// let conflicts = graph.detect_version_conflicts();
    /// assert_eq!(conflicts.len(), 1);
    /// ```
    pub fn detect_version_conflicts(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();

        for (package_name, requirements) in &self.version_requirements {
            if requirements.len() <= 1 {
                continue;
            }

            // Check if there are different versions requested
            let versions: HashSet<&str> = requirements.iter().map(|r| r.version.as_str()).collect();

            if versions.len() > 1 {
                conflicts.push(VersionConflict {
                    package_name: package_name.clone(),
                    requirements: requirements.clone(),
                });
            }
        }

        conflicts
    }

    /// Returns a set of package names that have version conflicts.
    ///
    /// This is useful for marking nodes in the UI that have conflicting versions.
    ///
    /// # Returns
    ///
    /// A `HashSet` of package names with version conflicts.
    pub fn get_packages_with_conflicts(&self) -> HashSet<String> {
        self.detect_version_conflicts()
            .into_iter()
            .map(|c| c.package_name)
            .collect()
    }

    /// Checks if any version conflicts exist.
    ///
    /// # Returns
    ///
    /// `true` if there are any version conflicts, `false` otherwise.
    pub fn has_version_conflicts(&self) -> bool {
        !self.detect_version_conflicts().is_empty()
    }

    /// Returns the number of nodes in the graph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// assert_eq!(graph.node_count(), 0);
    ///
    /// graph.add_dependency("react", "18.2.0", DependencyType::Production);
    /// assert_eq!(graph.node_count(), 1);
    /// ```
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns the number of edges in the graph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("a", "1.0.0", DependencyType::Production);
    /// graph.add_dependency("b", "1.0.0", DependencyType::Production);
    /// assert_eq!(graph.edge_count(), 0);
    ///
    /// graph.add_edge("a", "b");
    /// assert_eq!(graph.edge_count(), 1);
    /// ```
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Checks if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Checks if a node exists in the graph.
    pub fn contains(&self, name: &str) -> bool {
        self.node_indices.contains_key(name)
    }

    /// Gets nodes filtered by dependency type.
    ///
    /// # Arguments
    ///
    /// * `dep_type` - The dependency type to filter by
    ///
    /// # Returns
    ///
    /// A vector of references to nodes matching the specified type.
    pub fn get_nodes_by_type(&self, dep_type: DependencyType) -> Vec<&DependencyNode> {
        self.graph
            .node_weights()
            .filter(|node| node.dep_type == dep_type)
            .collect()
    }

    /// Gets nodes at a specific depth.
    ///
    /// # Arguments
    ///
    /// * `depth` - The depth level (0 = direct dependencies)
    ///
    /// # Returns
    ///
    /// A vector of references to nodes at the specified depth.
    pub fn get_nodes_at_depth(&self, depth: usize) -> Vec<&DependencyNode> {
        self.graph
            .node_weights()
            .filter(|node| node.depth == depth)
            .collect()
    }
}

/// A simple dependency structure for building graphs from parsed data.
///
/// This serves as an intermediate representation that can be converted
/// into graph nodes.
#[derive(Debug, Clone)]
pub struct Dependency {
    /// Package name
    pub name: String,
    /// Version specification
    pub version: String,
    /// Type of dependency
    pub dep_type: DependencyType,
}

/// Information about a detected circular dependency cycle.
///
/// Contains the list of package names that form the cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleInfo {
    /// The package names in the cycle (the last connects back to the first)
    pub nodes: Vec<String>,
}

impl CycleInfo {
    /// Returns a formatted string representation of the cycle path.
    ///
    /// For example: "a -> b -> c -> a"
    pub fn cycle_path(&self) -> String {
        if self.nodes.is_empty() {
            return String::new();
        }
        let mut path = self.nodes.join(" -> ");
        if !self.nodes.is_empty() {
            path.push_str(" -> ");
            path.push_str(&self.nodes[0]);
        }
        path
    }

    /// Returns the number of packages in the cycle.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the cycle is empty (should not happen in practice).
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Represents a version requirement from a specific package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRequirement {
    /// The version specification (e.g., "^1.0.0", ">=2.0.0")
    pub version: String,
    /// The package that requires this version
    pub required_by: String,
}

impl VersionRequirement {
    /// Creates a new version requirement.
    pub fn new(version: impl Into<String>, required_by: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            required_by: required_by.into(),
        }
    }
}

/// Information about a version conflict for a package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionConflict {
    /// The package name with conflicting versions
    pub package_name: String,
    /// All different version requirements for this package
    pub requirements: Vec<VersionRequirement>,
}

impl VersionConflict {
    /// Returns a formatted string describing the conflict.
    ///
    /// For example: "lodash requires: ^4.17.0 (by my-app), ^4.16.0 (by other-pkg)"
    pub fn description(&self) -> String {
        let reqs: Vec<String> = self
            .requirements
            .iter()
            .map(|r| format!("{} (by {})", r.version, r.required_by))
            .collect();
        format!("{} requires: {}", self.package_name, reqs.join(", "))
    }

    /// Returns the number of conflicting requirements.
    pub fn len(&self) -> usize {
        self.requirements.len()
    }

    /// Returns true if there are no requirements (should not happen).
    pub fn is_empty(&self) -> bool {
        self.requirements.is_empty()
    }
}

impl Dependency {
    /// Creates a new dependency.
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
}

impl DependencyGraph {
    /// Creates a graph from a list of dependencies.
    ///
    /// This is a convenience method for building graphs from parsed data.
    /// Note that this only adds nodes; edges should be added separately
    /// based on resolved dependency relationships.
    ///
    /// # Arguments
    ///
    /// * `deps` - A vector of dependencies to add
    ///
    /// # Returns
    ///
    /// A new `DependencyGraph` containing all dependencies as nodes.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, Dependency, DependencyType};
    ///
    /// let deps = vec![
    ///     Dependency { name: "react".into(), version: "18.2.0".into(), dep_type: DependencyType::Production },
    ///     Dependency { name: "typescript".into(), version: "5.0.0".into(), dep_type: DependencyType::Development },
    /// ];
    ///
    /// let graph = DependencyGraph::from_dependencies(deps);
    /// assert_eq!(graph.node_count(), 2);
    /// ```
    pub fn from_dependencies(deps: Vec<Dependency>) -> Self {
        let mut graph = Self::with_capacity(deps.len(), deps.len());

        for dep in deps {
            graph.add_dependency(&dep.name, &dep.version, dep.dep_type);
        }

        graph
    }

    /// Applies bundle size information to nodes in the graph.
    ///
    /// Takes a map of package names to their bundle size and module count,
    /// and updates the corresponding nodes in the graph.
    ///
    /// # Arguments
    ///
    /// * `sizes` - A map from package name to (size_in_bytes, module_count)
    ///
    /// # Returns
    ///
    /// The number of nodes that were updated with size information.
    ///
    /// # Example
    ///
    /// ```rust
    /// use codescope::graph::{DependencyGraph, DependencyType};
    /// use std::collections::HashMap;
    ///
    /// let mut graph = DependencyGraph::new();
    /// graph.add_dependency("react", "18.2.0", DependencyType::Production);
    /// graph.add_dependency("lodash", "4.17.21", DependencyType::Production);
    ///
    /// let mut sizes = HashMap::new();
    /// sizes.insert("react".to_string(), (10000_u64, 5_usize));
    /// sizes.insert("lodash".to_string(), (25000_u64, 10_usize));
    ///
    /// let updated = graph.apply_bundle_sizes(&sizes);
    /// assert_eq!(updated, 2);
    /// ```
    pub fn apply_bundle_sizes(&mut self, sizes: &HashMap<String, (u64, usize)>) -> usize {
        let mut updated = 0;

        for (name, &(size, module_count)) in sizes {
            if let Some(&idx) = self.node_indices.get(name) {
                if let Some(node) = self.graph.node_weight_mut(idx) {
                    node.set_bundle_size(size, module_count);
                    updated += 1;
                }
            }
        }

        updated
    }

    /// Gets a mutable reference to a dependency node by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Package name to look up
    ///
    /// # Returns
    ///
    /// `Some(&mut DependencyNode)` if found, `None` otherwise.
    pub fn get_node_mut(&mut self, name: &str) -> Option<&mut DependencyNode> {
        self.node_indices
            .get(name)
            .and_then(|&idx| self.graph.node_weight_mut(idx))
    }

    /// Gets all nodes with bundle size information.
    ///
    /// # Returns
    ///
    /// A vector of references to nodes that have bundle size data.
    pub fn get_nodes_with_sizes(&self) -> Vec<&DependencyNode> {
        self.graph
            .node_weights()
            .filter(|node| node.has_bundle_size())
            .collect()
    }

    /// Gets nodes sorted by bundle size (largest first).
    ///
    /// Only includes nodes that have bundle size information.
    ///
    /// # Returns
    ///
    /// A vector of references to nodes sorted by bundle size in descending order.
    pub fn get_nodes_by_bundle_size(&self) -> Vec<&DependencyNode> {
        let mut nodes: Vec<_> = self.get_nodes_with_sizes();
        nodes.sort_by(|a, b| {
            b.bundle_size
                .unwrap_or(0)
                .cmp(&a.bundle_size.unwrap_or(0))
        });
        nodes
    }

    /// Calculates the total bundle size of all dependencies.
    ///
    /// # Returns
    ///
    /// The sum of all known bundle sizes in bytes.
    pub fn total_bundle_size(&self) -> u64 {
        self.graph
            .node_weights()
            .filter_map(|node| node.bundle_size)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_graph() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        let idx = graph.add_dependency("react", "18.2.0", DependencyType::Production);

        assert_eq!(graph.node_count(), 1);
        assert!(graph.contains("react"));

        // Adding same dependency should return same index
        let idx2 = graph.add_dependency("react", "18.2.0", DependencyType::Production);
        assert_eq!(idx, idx2);
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_add_dependency_with_depth() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency_with_depth("react", "18.2.0", DependencyType::Production, 0);
        graph.add_dependency_with_depth("scheduler", "0.23.0", DependencyType::Production, 1);

        let direct = graph.get_nodes_at_depth(0);
        assert_eq!(direct.len(), 1);
        assert_eq!(direct[0].name, "react");

        let transitive = graph.get_nodes_at_depth(1);
        assert_eq!(transitive.len(), 1);
        assert_eq!(transitive[0].name, "scheduler");
    }

    #[test]
    fn test_get_node() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.2.0", DependencyType::Production);

        let node = graph.get_node("react");
        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.name, "react");
        assert_eq!(node.version, "18.2.0");
        assert_eq!(node.dep_type, DependencyType::Production);

        assert!(graph.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react-dom", "18.2.0", DependencyType::Production);
        graph.add_dependency("react", "18.2.0", DependencyType::Production);

        assert!(graph.add_edge("react-dom", "react"));
        assert_eq!(graph.edge_count(), 1);

        // Adding edge with nonexistent node should fail
        assert!(!graph.add_edge("nonexistent", "react"));
        assert!(!graph.add_edge("react", "nonexistent"));
    }

    #[test]
    fn test_get_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("my-app", "1.0.0", DependencyType::Production);
        graph.add_dependency("react", "18.2.0", DependencyType::Production);
        graph.add_dependency("lodash", "4.17.21", DependencyType::Production);

        graph.add_edge("my-app", "react");
        graph.add_edge("my-app", "lodash");

        let deps = graph.get_dependencies("my-app");
        assert_eq!(deps.len(), 2);

        let dep_names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(dep_names.contains(&"react"));
        assert!(dep_names.contains(&"lodash"));

        // Non-existent node returns empty
        assert!(graph.get_dependencies("nonexistent").is_empty());
    }

    #[test]
    fn test_get_dependents() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.2.0", DependencyType::Production);
        graph.add_dependency("react-dom", "18.2.0", DependencyType::Production);
        graph.add_dependency("my-app", "1.0.0", DependencyType::Production);

        graph.add_edge("react-dom", "react");
        graph.add_edge("my-app", "react");

        let dependents = graph.get_dependents("react");
        assert_eq!(dependents.len(), 2);

        let dependent_names: Vec<&str> = dependents.iter().map(|d| d.name.as_str()).collect();
        assert!(dependent_names.contains(&"react-dom"));
        assert!(dependent_names.contains(&"my-app"));
    }

    #[test]
    fn test_get_all_nodes() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.2.0", DependencyType::Production);
        graph.add_dependency("typescript", "5.0.0", DependencyType::Development);

        let nodes = graph.get_all_nodes();
        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_get_nodes_by_type() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.2.0", DependencyType::Production);
        graph.add_dependency("typescript", "5.0.0", DependencyType::Development);
        graph.add_dependency("eslint", "8.0.0", DependencyType::Development);

        let prod_deps = graph.get_nodes_by_type(DependencyType::Production);
        assert_eq!(prod_deps.len(), 1);
        assert_eq!(prod_deps[0].name, "react");

        let dev_deps = graph.get_nodes_by_type(DependencyType::Development);
        assert_eq!(dev_deps.len(), 2);
    }

    #[test]
    fn test_has_cycles_no_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", DependencyType::Production);
        graph.add_dependency("b", "1.0.0", DependencyType::Production);
        graph.add_dependency("c", "1.0.0", DependencyType::Production);

        graph.add_edge("a", "b");
        graph.add_edge("b", "c");

        assert!(!graph.has_cycles());
    }

    #[test]
    fn test_has_cycles_with_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", DependencyType::Production);
        graph.add_dependency("b", "1.0.0", DependencyType::Production);

        graph.add_edge("a", "b");
        graph.add_edge("b", "a"); // Creates cycle

        assert!(graph.has_cycles());
    }

    #[test]
    fn test_detect_cycles() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", DependencyType::Production);
        graph.add_dependency("b", "1.0.0", DependencyType::Production);
        graph.add_dependency("c", "1.0.0", DependencyType::Production);

        graph.add_edge("a", "b");
        graph.add_edge("b", "c");
        graph.add_edge("c", "a"); // Creates cycle: a -> b -> c -> a

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());

        // The cycle should contain all three nodes
        let cycle = &cycles[0];
        assert_eq!(cycle.len(), 3);
        assert!(cycle.contains(&"a".to_string()));
        assert!(cycle.contains(&"b".to_string()));
        assert!(cycle.contains(&"c".to_string()));
    }

    #[test]
    fn test_detect_cycles_self_loop() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("self-ref", "1.0.0", DependencyType::Production);
        graph.add_edge("self-ref", "self-ref");

        let cycles = graph.detect_cycles();
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0], vec!["self-ref"]);
    }

    #[test]
    fn test_from_dependencies() {
        let deps = vec![
            Dependency::new("react", "18.2.0", DependencyType::Production),
            Dependency::new("typescript", "5.0.0", DependencyType::Development),
            Dependency::new("@types/react", "18.0.0", DependencyType::Development),
        ];

        let graph = DependencyGraph::from_dependencies(deps);

        assert_eq!(graph.node_count(), 3);
        assert!(graph.contains("react"));
        assert!(graph.contains("typescript"));
        assert!(graph.contains("@types/react"));
    }

    #[test]
    fn test_optional_edge() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("my-app", "1.0.0", DependencyType::Production);
        graph.add_dependency("optional-dep", "1.0.0", DependencyType::Optional);

        assert!(graph.add_optional_edge("my-app", "optional-dep"));
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_dependency_type_display() {
        assert_eq!(format!("{}", DependencyType::Production), "production");
        assert_eq!(format!("{}", DependencyType::Development), "dev");
        assert_eq!(format!("{}", DependencyType::Peer), "peer");
        assert_eq!(format!("{}", DependencyType::Optional), "optional");
    }

    #[test]
    fn test_default_dependency_type() {
        let default = DependencyType::default();
        assert_eq!(default, DependencyType::Production);
    }

    #[test]
    fn test_get_nodes_in_cycles() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", DependencyType::Production);
        graph.add_dependency("b", "1.0.0", DependencyType::Production);
        graph.add_dependency("c", "1.0.0", DependencyType::Production);
        graph.add_dependency("d", "1.0.0", DependencyType::Production);

        graph.add_edge("a", "b");
        graph.add_edge("b", "c");
        graph.add_edge("c", "a"); // Creates cycle: a -> b -> c -> a
        graph.add_edge("a", "d"); // d is not part of the cycle

        let cycle_nodes = graph.get_nodes_in_cycles();
        assert!(cycle_nodes.contains("a"));
        assert!(cycle_nodes.contains("b"));
        assert!(cycle_nodes.contains("c"));
        assert!(!cycle_nodes.contains("d")); // d is not in the cycle
        assert_eq!(cycle_nodes.len(), 3);
    }

    #[test]
    fn test_get_nodes_in_cycles_no_cycles() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", DependencyType::Production);
        graph.add_dependency("b", "1.0.0", DependencyType::Production);
        graph.add_dependency("c", "1.0.0", DependencyType::Production);

        graph.add_edge("a", "b");
        graph.add_edge("b", "c");

        let cycle_nodes = graph.get_nodes_in_cycles();
        assert!(cycle_nodes.is_empty());
    }

    #[test]
    fn test_get_cycle_details() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "1.0.0", DependencyType::Production);
        graph.add_dependency("b", "1.0.0", DependencyType::Production);
        graph.add_dependency("c", "1.0.0", DependencyType::Production);

        graph.add_edge("a", "b");
        graph.add_edge("b", "c");
        graph.add_edge("c", "a"); // Creates cycle

        let cycle_details = graph.get_cycle_details();
        assert_eq!(cycle_details.len(), 1);
        assert_eq!(cycle_details[0].len(), 3);
    }

    #[test]
    fn test_cycle_info_cycle_path() {
        let cycle = CycleInfo {
            nodes: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        assert_eq!(cycle.cycle_path(), "a -> b -> c -> a");
    }

    #[test]
    fn test_cycle_info_empty() {
        let cycle = CycleInfo { nodes: vec![] };
        assert!(cycle.is_empty());
        assert_eq!(cycle.len(), 0);
        assert_eq!(cycle.cycle_path(), "");
    }

    #[test]
    fn test_multiple_cycles() {
        let mut graph = DependencyGraph::new();
        // First cycle: a -> b -> a
        graph.add_dependency("a", "1.0.0", DependencyType::Production);
        graph.add_dependency("b", "1.0.0", DependencyType::Production);
        graph.add_edge("a", "b");
        graph.add_edge("b", "a");

        // Second cycle: c -> d -> e -> c
        graph.add_dependency("c", "1.0.0", DependencyType::Production);
        graph.add_dependency("d", "1.0.0", DependencyType::Production);
        graph.add_dependency("e", "1.0.0", DependencyType::Production);
        graph.add_edge("c", "d");
        graph.add_edge("d", "e");
        graph.add_edge("e", "c");

        let cycle_nodes = graph.get_nodes_in_cycles();
        assert_eq!(cycle_nodes.len(), 5);
        assert!(cycle_nodes.contains("a"));
        assert!(cycle_nodes.contains("b"));
        assert!(cycle_nodes.contains("c"));
        assert!(cycle_nodes.contains("d"));
        assert!(cycle_nodes.contains("e"));
    }

    // Version conflict tests
    #[test]
    fn test_track_version_requirement() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.track_version_requirement("lodash", "^4.17.0", "my-app");
        graph.track_version_requirement("lodash", "^4.17.0", "other-pkg");

        // Same version from different packages should not be a conflict
        assert!(!graph.has_version_conflicts());
    }

    #[test]
    fn test_detect_version_conflicts() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.track_version_requirement("lodash", "^4.17.0", "my-app");
        graph.track_version_requirement("lodash", "^4.16.0", "other-pkg");

        let conflicts = graph.detect_version_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].package_name, "lodash");
        assert_eq!(conflicts[0].requirements.len(), 2);
    }

    #[test]
    fn test_no_version_conflicts() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.track_version_requirement("lodash", "^4.17.0", "my-app");
        graph.track_version_requirement("lodash", "^4.17.0", "other-pkg");

        let conflicts = graph.detect_version_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_get_packages_with_conflicts() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.add_dependency("react", "18.0.0", DependencyType::Production);

        graph.track_version_requirement("lodash", "^4.17.0", "my-app");
        graph.track_version_requirement("lodash", "^4.16.0", "other-pkg");
        graph.track_version_requirement("react", "^18.0.0", "my-app");
        graph.track_version_requirement("react", "^18.0.0", "another-pkg");

        let conflicts = graph.get_packages_with_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts.contains("lodash"));
        assert!(!conflicts.contains("react")); // Same version, no conflict
    }

    #[test]
    fn test_multiple_version_conflicts() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.add_dependency("react", "18.0.0", DependencyType::Production);

        graph.track_version_requirement("lodash", "^4.17.0", "app-a");
        graph.track_version_requirement("lodash", "^4.16.0", "app-b");
        graph.track_version_requirement("react", "^18.0.0", "app-a");
        graph.track_version_requirement("react", "^17.0.0", "app-b");

        let conflicts = graph.detect_version_conflicts();
        assert_eq!(conflicts.len(), 2);

        let conflict_names: HashSet<String> =
            conflicts.iter().map(|c| c.package_name.clone()).collect();
        assert!(conflict_names.contains("lodash"));
        assert!(conflict_names.contains("react"));
    }

    #[test]
    fn test_version_conflict_description() {
        let conflict = VersionConflict {
            package_name: "lodash".to_string(),
            requirements: vec![
                VersionRequirement::new("^4.17.0", "my-app"),
                VersionRequirement::new("^4.16.0", "other-pkg"),
            ],
        };
        let desc = conflict.description();
        assert!(desc.contains("lodash"));
        assert!(desc.contains("^4.17.0"));
        assert!(desc.contains("my-app"));
        assert!(desc.contains("^4.16.0"));
        assert!(desc.contains("other-pkg"));
    }

    #[test]
    fn test_version_requirement_new() {
        let req = VersionRequirement::new("^4.17.0", "my-app");
        assert_eq!(req.version, "^4.17.0");
        assert_eq!(req.required_by, "my-app");
    }

    // Bundle size tests
    #[test]
    fn test_dependency_node_with_bundle_size() {
        let node = DependencyNode::with_bundle_size(
            "react",
            "18.0.0",
            DependencyType::Production,
            10000,
            5,
        );
        assert_eq!(node.name, "react");
        assert_eq!(node.bundle_size, Some(10000));
        assert_eq!(node.module_count, Some(5));
        assert!(node.has_bundle_size());
    }

    #[test]
    fn test_dependency_node_set_bundle_size() {
        let mut node = DependencyNode::new("react", "18.0.0", DependencyType::Production);
        assert!(!node.has_bundle_size());
        assert_eq!(node.bundle_size, None);

        node.set_bundle_size(5000, 3);
        assert!(node.has_bundle_size());
        assert_eq!(node.bundle_size, Some(5000));
        assert_eq!(node.module_count, Some(3));
    }

    #[test]
    fn test_apply_bundle_sizes() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.0.0", DependencyType::Production);
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.add_dependency("typescript", "5.0.0", DependencyType::Development);

        let mut sizes = HashMap::new();
        sizes.insert("react".to_string(), (10000_u64, 5_usize));
        sizes.insert("lodash".to_string(), (25000_u64, 10_usize));

        let updated = graph.apply_bundle_sizes(&sizes);

        assert_eq!(updated, 2);
        assert_eq!(graph.get_node("react").unwrap().bundle_size, Some(10000));
        assert_eq!(graph.get_node("react").unwrap().module_count, Some(5));
        assert_eq!(graph.get_node("lodash").unwrap().bundle_size, Some(25000));
        assert_eq!(graph.get_node("lodash").unwrap().module_count, Some(10));
        assert_eq!(graph.get_node("typescript").unwrap().bundle_size, None);
    }

    #[test]
    fn test_get_node_mut() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.0.0", DependencyType::Production);

        if let Some(node) = graph.get_node_mut("react") {
            node.set_bundle_size(5000, 3);
        }

        assert_eq!(graph.get_node("react").unwrap().bundle_size, Some(5000));
    }

    #[test]
    fn test_get_nodes_with_sizes() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.0.0", DependencyType::Production);
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.add_dependency("typescript", "5.0.0", DependencyType::Development);

        let mut sizes = HashMap::new();
        sizes.insert("react".to_string(), (10000_u64, 5_usize));
        graph.apply_bundle_sizes(&sizes);

        let nodes_with_sizes = graph.get_nodes_with_sizes();
        assert_eq!(nodes_with_sizes.len(), 1);
        assert_eq!(nodes_with_sizes[0].name, "react");
    }

    #[test]
    fn test_get_nodes_by_bundle_size() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("small", "1.0.0", DependencyType::Production);
        graph.add_dependency("large", "1.0.0", DependencyType::Production);
        graph.add_dependency("medium", "1.0.0", DependencyType::Production);
        graph.add_dependency("no-size", "1.0.0", DependencyType::Production);

        let mut sizes = HashMap::new();
        sizes.insert("small".to_string(), (1000_u64, 1_usize));
        sizes.insert("large".to_string(), (50000_u64, 20_usize));
        sizes.insert("medium".to_string(), (10000_u64, 5_usize));
        graph.apply_bundle_sizes(&sizes);

        let sorted = graph.get_nodes_by_bundle_size();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].name, "large");
        assert_eq!(sorted[1].name, "medium");
        assert_eq!(sorted[2].name, "small");
    }

    #[test]
    fn test_total_bundle_size() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("react", "18.0.0", DependencyType::Production);
        graph.add_dependency("lodash", "4.17.0", DependencyType::Production);
        graph.add_dependency("no-size", "1.0.0", DependencyType::Production);

        let mut sizes = HashMap::new();
        sizes.insert("react".to_string(), (10000_u64, 5_usize));
        sizes.insert("lodash".to_string(), (25000_u64, 10_usize));
        graph.apply_bundle_sizes(&sizes);

        assert_eq!(graph.total_bundle_size(), 35000);
    }

    #[test]
    fn test_total_bundle_size_empty() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.total_bundle_size(), 0);
    }
}
