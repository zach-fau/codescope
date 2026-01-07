//! Tree data structures for dependency visualization
//!
//! Provides `TreeNode` for hierarchical data and `FlattenedNode`
//! for rendering the tree as a scrollable list in the TUI.

use crate::parser::types::DependencyType;
use std::collections::HashSet;

/// A node in the dependency tree
#[derive(Debug, Clone)]
pub struct TreeNode {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Child dependencies
    pub children: Vec<TreeNode>,
    /// Whether this node is expanded in the UI
    pub expanded: bool,
    /// Depth in the tree (0 = root)
    pub depth: usize,
    /// The type of dependency (Production, Development, Peer, Optional)
    pub dep_type: Option<DependencyType>,
    /// Whether this node is part of a circular dependency
    pub is_in_cycle: bool,
    /// Whether this node has a version conflict
    pub has_conflict: bool,
    /// Bundle size in bytes (from webpack/bundler stats)
    pub bundle_size: Option<u64>,
    /// Number of modules from this package included in the bundle
    pub module_count: Option<usize>,
}

impl TreeNode {
    /// Create a new tree node
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            children: Vec::new(),
            expanded: false,
            depth: 0,
            dep_type: None,
            is_in_cycle: false,
            has_conflict: false,
            bundle_size: None,
            module_count: None,
        }
    }

    /// Create a new tree node with specified depth
    #[cfg(test)]
    pub fn with_depth(name: String, version: String, depth: usize) -> Self {
        Self {
            name,
            version,
            children: Vec::new(),
            expanded: false,
            depth,
            dep_type: None,
            is_in_cycle: false,
            has_conflict: false,
            bundle_size: None,
            module_count: None,
        }
    }

    /// Create a new tree node with dependency type
    pub fn with_dep_type(name: String, version: String, dep_type: DependencyType) -> Self {
        Self {
            name,
            version,
            children: Vec::new(),
            expanded: false,
            depth: 0,
            dep_type: Some(dep_type),
            is_in_cycle: false,
            has_conflict: false,
            bundle_size: None,
            module_count: None,
        }
    }

    /// Create a new tree node with bundle size information
    pub fn with_bundle_size(
        name: String,
        version: String,
        bundle_size: u64,
        module_count: usize,
    ) -> Self {
        Self {
            name,
            version,
            children: Vec::new(),
            expanded: false,
            depth: 0,
            dep_type: None,
            is_in_cycle: false,
            has_conflict: false,
            bundle_size: Some(bundle_size),
            module_count: Some(module_count),
        }
    }

    /// Set bundle size information for this node
    pub fn set_bundle_size(&mut self, size: u64, module_count: usize) {
        self.bundle_size = Some(size);
        self.module_count = Some(module_count);
    }

    /// Returns true if this node has bundle size information
    pub fn has_bundle_size(&self) -> bool {
        self.bundle_size.is_some()
    }

    /// Apply bundle sizes from a map to this node and all children recursively
    pub fn apply_bundle_sizes(&mut self, sizes: &std::collections::HashMap<String, (u64, usize)>) {
        if let Some(&(size, count)) = sizes.get(&self.name) {
            self.bundle_size = Some(size);
            self.module_count = Some(count);
        }
        for child in &mut self.children {
            child.apply_bundle_sizes(sizes);
        }
    }

    /// Mark nodes that are part of cycles based on a set of cycle node names.
    ///
    /// This method recursively marks all nodes in the tree that match
    /// names in the provided set.
    pub fn mark_cycles(&mut self, cycle_nodes: &HashSet<String>) {
        self.is_in_cycle = cycle_nodes.contains(&self.name);
        for child in &mut self.children {
            child.mark_cycles(cycle_nodes);
        }
    }

    /// Mark nodes that have version conflicts based on a set of conflicting package names.
    ///
    /// This method recursively marks all nodes in the tree that match
    /// names in the provided set.
    pub fn mark_conflicts(&mut self, conflict_packages: &HashSet<String>) {
        self.has_conflict = conflict_packages.contains(&self.name);
        for child in &mut self.children {
            child.mark_conflicts(conflict_packages);
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, mut child: TreeNode) {
        child.depth = self.depth + 1;
        self.children.push(child);
    }

    /// Toggle the expanded state
    pub fn toggle_expanded(&mut self) {
        if !self.children.is_empty() {
            self.expanded = !self.expanded;
        }
    }

    /// Check if this node has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Flatten the tree into a list for rendering
    ///
    /// Only includes nodes that are visible (i.e., all ancestors are expanded)
    pub fn flatten(&self) -> Vec<FlattenedNode> {
        let mut result = Vec::new();
        self.flatten_recursive(&mut result, true);
        result
    }

    fn flatten_recursive(&self, result: &mut Vec<FlattenedNode>, is_last: bool) {
        result.push(FlattenedNode {
            name: self.name.clone(),
            version: self.version.clone(),
            depth: self.depth,
            is_expanded: self.expanded,
            has_children: self.has_children(),
            is_last_child: is_last,
            dep_type: self.dep_type,
            is_in_cycle: self.is_in_cycle,
            has_conflict: self.has_conflict,
            bundle_size: self.bundle_size,
            module_count: self.module_count,
        });

        if self.expanded {
            let child_count = self.children.len();
            for (i, child) in self.children.iter().enumerate() {
                let is_last_child = i == child_count - 1;
                child.flatten_recursive(result, is_last_child);
            }
        }
    }

    /// Find a node at a given flattened index and toggle its expansion
    ///
    /// Returns true if the toggle was successful
    pub fn toggle_at_index(&mut self, target_index: usize) -> bool {
        let mut current_index = 0;
        self.toggle_at_index_recursive(target_index, &mut current_index)
    }

    fn toggle_at_index_recursive(
        &mut self,
        target_index: usize,
        current_index: &mut usize,
    ) -> bool {
        if *current_index == target_index {
            self.toggle_expanded();
            return true;
        }
        *current_index += 1;

        if self.expanded {
            for child in &mut self.children {
                if child.toggle_at_index_recursive(target_index, current_index) {
                    return true;
                }
            }
        }
        false
    }
}

/// A flattened representation of a tree node for rendering
#[derive(Debug, Clone)]
pub struct FlattenedNode {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Depth in the tree
    pub depth: usize,
    /// Whether this node is currently expanded
    pub is_expanded: bool,
    /// Whether this node has children
    pub has_children: bool,
    /// Whether this is the last child of its parent
    pub is_last_child: bool,
    /// The type of dependency (Production, Development, Peer, Optional)
    pub dep_type: Option<DependencyType>,
    /// Whether this node is part of a circular dependency
    pub is_in_cycle: bool,
    /// Whether this node has a version conflict
    pub has_conflict: bool,
    /// Bundle size in bytes (from webpack/bundler stats)
    pub bundle_size: Option<u64>,
    /// Number of modules from this package included in the bundle
    pub module_count: Option<usize>,
}

impl FlattenedNode {
    /// Get the expansion indicator character
    pub fn expansion_indicator(&self) -> &'static str {
        if !self.has_children {
            "  "
        } else if self.is_expanded {
            "▼ "
        } else {
            "▶ "
        }
    }

    /// Build the tree prefix (indentation and branch lines)
    #[allow(dead_code)]
    pub fn tree_prefix(&self, ancestors_are_last: &[bool]) -> String {
        let mut prefix = String::new();

        // Add indentation for each ancestor level
        for (i, &is_last) in ancestors_are_last.iter().enumerate() {
            if i < self.depth {
                if is_last {
                    prefix.push_str("    ");
                } else {
                    prefix.push_str("│   ");
                }
            }
        }

        // Add the branch connector for this node
        if self.depth > 0 {
            if self.is_last_child {
                prefix.push_str("└── ");
            } else {
                prefix.push_str("├── ");
            }
        }

        prefix
    }

    /// Returns true if this node has bundle size information
    pub fn has_bundle_size(&self) -> bool {
        self.bundle_size.is_some()
    }

    /// Format the bundle size as a human-readable string
    pub fn format_bundle_size(&self) -> Option<String> {
        self.bundle_size.map(format_size)
    }
}

/// Format a byte size as a human-readable string.
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

    fn create_test_tree() -> TreeNode {
        let mut root = TreeNode::new("project".to_string(), "1.0.0".to_string());

        let mut dep_a = TreeNode::with_depth("dep-a".to_string(), "2.0.0".to_string(), 1);
        dep_a.add_child(TreeNode::new("sub-dep-1".to_string(), "0.1.0".to_string()));
        dep_a.add_child(TreeNode::new("sub-dep-2".to_string(), "0.2.0".to_string()));

        let dep_b = TreeNode::with_depth("dep-b".to_string(), "3.0.0".to_string(), 1);

        root.children.push(dep_a);
        root.children.push(dep_b);

        root
    }

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new("test".to_string(), "1.0.0".to_string());
        assert_eq!(node.name, "test");
        assert_eq!(node.version, "1.0.0");
        assert!(!node.expanded);
        assert_eq!(node.depth, 0);
    }

    #[test]
    fn test_add_child() {
        let mut parent = TreeNode::new("parent".to_string(), "1.0.0".to_string());
        let child = TreeNode::new("child".to_string(), "0.1.0".to_string());
        parent.add_child(child);

        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].depth, 1);
    }

    #[test]
    fn test_flatten_collapsed() {
        let root = create_test_tree();
        let flattened = root.flatten();

        // Only root should be visible when collapsed
        assert_eq!(flattened.len(), 1);
        assert_eq!(flattened[0].name, "project");
    }

    #[test]
    fn test_flatten_expanded() {
        let mut root = create_test_tree();
        root.expanded = true;
        let flattened = root.flatten();

        // Root + 2 children should be visible
        assert_eq!(flattened.len(), 3);
        assert_eq!(flattened[0].name, "project");
        assert_eq!(flattened[1].name, "dep-a");
        assert_eq!(flattened[2].name, "dep-b");
    }

    #[test]
    fn test_flatten_fully_expanded() {
        let mut root = create_test_tree();
        root.expanded = true;
        root.children[0].expanded = true;
        let flattened = root.flatten();

        // All nodes should be visible
        assert_eq!(flattened.len(), 5);
    }

    #[test]
    fn test_toggle_at_index() {
        let mut root = create_test_tree();
        root.expanded = true;

        // Toggle dep-a (index 1)
        assert!(root.toggle_at_index(1));
        assert!(root.children[0].expanded);

        // Toggle again
        assert!(root.toggle_at_index(1));
        assert!(!root.children[0].expanded);
    }

    #[test]
    fn test_expansion_indicator() {
        let node_with_children = FlattenedNode {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            depth: 0,
            is_expanded: false,
            has_children: true,
            is_last_child: false,
            dep_type: None,
            is_in_cycle: false,
            has_conflict: false,
            bundle_size: None,
            module_count: None,
        };
        assert_eq!(node_with_children.expansion_indicator(), "▶ ");

        let expanded_node = FlattenedNode {
            is_expanded: true,
            ..node_with_children.clone()
        };
        assert_eq!(expanded_node.expansion_indicator(), "▼ ");

        let leaf_node = FlattenedNode {
            has_children: false,
            ..node_with_children
        };
        assert_eq!(leaf_node.expansion_indicator(), "  ");
    }

    #[test]
    fn test_tree_node_with_dep_type() {
        let node = TreeNode::with_dep_type(
            "react".to_string(),
            "18.0.0".to_string(),
            DependencyType::Production,
        );
        assert_eq!(node.name, "react");
        assert_eq!(node.dep_type, Some(DependencyType::Production));
    }

    #[test]
    fn test_mark_cycles() {
        let mut root = TreeNode::new("project".to_string(), "1.0.0".to_string());
        let dep_a = TreeNode::new("dep-a".to_string(), "1.0.0".to_string());
        let dep_b = TreeNode::new("dep-b".to_string(), "1.0.0".to_string());
        let dep_c = TreeNode::new("dep-c".to_string(), "1.0.0".to_string());

        root.add_child(dep_a);
        root.add_child(dep_b);
        root.add_child(dep_c);

        // Mark dep-a and dep-b as being in a cycle
        let mut cycle_nodes = HashSet::new();
        cycle_nodes.insert("dep-a".to_string());
        cycle_nodes.insert("dep-b".to_string());

        root.mark_cycles(&cycle_nodes);

        assert!(!root.is_in_cycle);
        assert!(root.children[0].is_in_cycle); // dep-a
        assert!(root.children[1].is_in_cycle); // dep-b
        assert!(!root.children[2].is_in_cycle); // dep-c
    }

    #[test]
    fn test_flatten_includes_cycle_info() {
        let mut root = TreeNode::new("project".to_string(), "1.0.0".to_string());
        let dep_a = TreeNode::new("dep-a".to_string(), "1.0.0".to_string());
        root.add_child(dep_a);

        let mut cycle_nodes = HashSet::new();
        cycle_nodes.insert("dep-a".to_string());

        root.mark_cycles(&cycle_nodes);
        root.expanded = true;

        let flattened = root.flatten();
        assert_eq!(flattened.len(), 2);
        assert!(!flattened[0].is_in_cycle); // project
        assert!(flattened[1].is_in_cycle); // dep-a
    }

    #[test]
    fn test_mark_conflicts() {
        let mut root = TreeNode::new("project".to_string(), "1.0.0".to_string());
        let dep_a = TreeNode::new("lodash".to_string(), "4.17.0".to_string());
        let dep_b = TreeNode::new("react".to_string(), "18.0.0".to_string());
        let dep_c = TreeNode::new("typescript".to_string(), "5.0.0".to_string());

        root.add_child(dep_a);
        root.add_child(dep_b);
        root.add_child(dep_c);

        // Mark lodash as having a conflict
        let mut conflict_packages = HashSet::new();
        conflict_packages.insert("lodash".to_string());

        root.mark_conflicts(&conflict_packages);

        assert!(!root.has_conflict);
        assert!(root.children[0].has_conflict); // lodash
        assert!(!root.children[1].has_conflict); // react
        assert!(!root.children[2].has_conflict); // typescript
    }

    #[test]
    fn test_flatten_includes_conflict_info() {
        let mut root = TreeNode::new("project".to_string(), "1.0.0".to_string());
        let dep_a = TreeNode::new("lodash".to_string(), "4.17.0".to_string());
        root.add_child(dep_a);

        let mut conflict_packages = HashSet::new();
        conflict_packages.insert("lodash".to_string());

        root.mark_conflicts(&conflict_packages);
        root.expanded = true;

        let flattened = root.flatten();
        assert_eq!(flattened.len(), 2);
        assert!(!flattened[0].has_conflict); // project
        assert!(flattened[1].has_conflict); // lodash
    }

    // Bundle size tests
    #[test]
    fn test_tree_node_with_bundle_size() {
        let node = TreeNode::with_bundle_size(
            "react".to_string(),
            "18.0.0".to_string(),
            10000,
            5,
        );
        assert_eq!(node.name, "react");
        assert_eq!(node.bundle_size, Some(10000));
        assert_eq!(node.module_count, Some(5));
        assert!(node.has_bundle_size());
    }

    #[test]
    fn test_tree_node_set_bundle_size() {
        let mut node = TreeNode::new("react".to_string(), "18.0.0".to_string());
        assert!(!node.has_bundle_size());
        assert_eq!(node.bundle_size, None);

        node.set_bundle_size(5000, 3);
        assert!(node.has_bundle_size());
        assert_eq!(node.bundle_size, Some(5000));
        assert_eq!(node.module_count, Some(3));
    }

    #[test]
    fn test_apply_bundle_sizes_to_tree() {
        let mut root = TreeNode::new("my-app".to_string(), "1.0.0".to_string());
        let react = TreeNode::new("react".to_string(), "18.0.0".to_string());
        let lodash = TreeNode::new("lodash".to_string(), "4.17.0".to_string());
        root.add_child(react);
        root.add_child(lodash);

        let mut sizes = std::collections::HashMap::new();
        sizes.insert("react".to_string(), (10000_u64, 5_usize));
        sizes.insert("lodash".to_string(), (25000_u64, 10_usize));

        root.apply_bundle_sizes(&sizes);

        assert_eq!(root.bundle_size, None);
        assert_eq!(root.children[0].bundle_size, Some(10000));
        assert_eq!(root.children[0].module_count, Some(5));
        assert_eq!(root.children[1].bundle_size, Some(25000));
        assert_eq!(root.children[1].module_count, Some(10));
    }

    #[test]
    fn test_apply_bundle_sizes_recursive() {
        let mut root = TreeNode::new("my-app".to_string(), "1.0.0".to_string());
        let mut react = TreeNode::new("react".to_string(), "18.0.0".to_string());
        let scheduler = TreeNode::new("scheduler".to_string(), "0.23.0".to_string());
        react.add_child(scheduler);
        root.add_child(react);

        let mut sizes = std::collections::HashMap::new();
        sizes.insert("react".to_string(), (10000_u64, 5_usize));
        sizes.insert("scheduler".to_string(), (500_u64, 2_usize));

        root.apply_bundle_sizes(&sizes);

        assert_eq!(root.children[0].bundle_size, Some(10000)); // react
        assert_eq!(root.children[0].children[0].bundle_size, Some(500)); // scheduler
    }

    #[test]
    fn test_flatten_includes_bundle_size() {
        let mut root = TreeNode::new("project".to_string(), "1.0.0".to_string());
        let mut react = TreeNode::new("react".to_string(), "18.0.0".to_string());
        react.bundle_size = Some(10000);
        react.module_count = Some(5);
        root.add_child(react);
        root.expanded = true;

        let flattened = root.flatten();
        assert_eq!(flattened.len(), 2);
        assert_eq!(flattened[0].bundle_size, None);
        assert_eq!(flattened[1].bundle_size, Some(10000));
        assert_eq!(flattened[1].module_count, Some(5));
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
    fn test_flattened_node_format_bundle_size() {
        let node = FlattenedNode {
            name: "react".to_string(),
            version: "18.0.0".to_string(),
            depth: 0,
            is_expanded: false,
            has_children: false,
            is_last_child: false,
            dep_type: None,
            is_in_cycle: false,
            has_conflict: false,
            bundle_size: Some(1048576),
            module_count: Some(5),
        };

        assert!(node.has_bundle_size());
        assert_eq!(node.format_bundle_size(), Some("1.00 MB".to_string()));
    }

    #[test]
    fn test_flattened_node_no_bundle_size() {
        let node = FlattenedNode {
            name: "react".to_string(),
            version: "18.0.0".to_string(),
            depth: 0,
            is_expanded: false,
            has_children: false,
            is_last_child: false,
            dep_type: None,
            is_in_cycle: false,
            has_conflict: false,
            bundle_size: None,
            module_count: None,
        };

        assert!(!node.has_bundle_size());
        assert_eq!(node.format_bundle_size(), None);
    }
}
