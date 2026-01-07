//! UI module for CodeScope TUI
//!
//! This module provides the terminal user interface for displaying
//! dependency trees and interacting with the analysis results.

mod app;
pub mod tree;

pub use app::{run_app, App};
pub use tree::TreeNode;
