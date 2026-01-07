//! Tree data structures for dependency visualization
//!
//! Provides `TreeNode` for hierarchical data and `FlattenedNode`
//! for rendering the tree as a scrollable list in the TUI.

use crate::parser::types::DependencyType;

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
}
