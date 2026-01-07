//! Graph module for dependency relationship modeling.
//!
//! This module provides the [`DependencyGraph`] struct for building and
//! analyzing dependency relationships using a directed graph structure.
//!
//! # Example
//!
//! ```rust
//! use codescope::graph::{DependencyGraph, DependencyType};
//!
//! let mut graph = DependencyGraph::new();
//! graph.add_dependency("react", "18.2.0", DependencyType::Production);
//! graph.add_dependency("react-dom", "18.2.0", DependencyType::Production);
//! graph.add_edge("react-dom", "react");
//!
//! assert_eq!(graph.node_count(), 2);
//! assert_eq!(graph.edge_count(), 1);
//! ```

mod dependency_graph;

pub use dependency_graph::{CycleInfo, DependencyEdge, DependencyGraph, DependencyNode, DependencyType};
