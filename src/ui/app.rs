//! Application state and TUI event loop
//!
//! Manages the application state and handles user input for the
//! dependency tree visualization.

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

use crate::parser::types::DependencyType;
use super::tree::{FlattenedNode, TreeNode};

/// Application state
pub struct App {
    /// The root of the dependency tree
    pub tree: TreeNode,
    /// Currently selected index in the flattened view
    pub selected_index: usize,
    /// Flattened representation for rendering
    pub flattened: Vec<FlattenedNode>,
    /// Track which ancestors are "last child" for proper tree drawing
    ancestors_last: Vec<bool>,
    /// Whether the application should quit
    pub should_quit: bool,
    /// List state for ratatui
    list_state: ListState,
}

impl App {
    /// Create a new application with the given root tree node
    pub fn new(root: TreeNode) -> Self {
        let mut app = Self {
            tree: root,
            selected_index: 0,
            flattened: Vec::new(),
            ancestors_last: Vec::new(),
            should_quit: false,
            list_state: ListState::default(),
        };
        app.refresh_flattened();
        app.list_state.select(Some(0));
        app
    }

    /// Refresh the flattened view from the tree
    pub fn refresh_flattened(&mut self) {
        self.flattened = self.tree.flatten();
        self.rebuild_ancestors_last();

        // Ensure selected index is valid
        if !self.flattened.is_empty() && self.selected_index >= self.flattened.len() {
            self.selected_index = self.flattened.len() - 1;
        }
    }

    /// Rebuild the ancestors_last tracking for tree drawing
    fn rebuild_ancestors_last(&mut self) {
        self.ancestors_last.clear();
        for node in &self.flattened {
            // Track the is_last_child status for each depth level
            while self.ancestors_last.len() < node.depth {
                self.ancestors_last.push(false);
            }
            if node.depth > 0 && self.ancestors_last.len() >= node.depth {
                self.ancestors_last[node.depth - 1] = node.is_last_child;
            }
        }
    }

    /// Move selection to the next item
    pub fn select_next(&mut self) {
        if !self.flattened.is_empty() {
            self.selected_index = (self.selected_index + 1).min(self.flattened.len() - 1);
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Move selection to the previous item
    pub fn select_previous(&mut self) {
        if !self.flattened.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
        }
    }

    /// Toggle expansion of the selected item
    pub fn toggle_selected(&mut self) {
        if self.tree.toggle_at_index(self.selected_index) {
            self.refresh_flattened();
        }
    }

    /// Signal that the application should quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Get the tree prefix for a node at the given index
    fn get_tree_prefix(&self, index: usize) -> String {
        if index >= self.flattened.len() {
            return String::new();
        }

        let node = &self.flattened[index];
        let mut prefix = String::new();

        // Build ancestors_last for this specific path
        let mut ancestors_last_for_node = Vec::new();
        let mut current_depth = 0;

        for (i, n) in self.flattened.iter().enumerate().take(index + 1) {
            if n.depth <= current_depth || i == index {
                while ancestors_last_for_node.len() > n.depth {
                    ancestors_last_for_node.pop();
                }
            }
            if n.depth > 0 {
                while ancestors_last_for_node.len() < n.depth {
                    ancestors_last_for_node.push(false);
                }
                if ancestors_last_for_node.len() >= n.depth {
                    ancestors_last_for_node[n.depth - 1] = n.is_last_child;
                }
            }
            current_depth = n.depth;
        }

        // Build the prefix string
        for i in 0..node.depth {
            if i < ancestors_last_for_node.len() {
                if ancestors_last_for_node[i] {
                    prefix.push_str("    ");
                } else {
                    prefix.push_str("│   ");
                }
            } else {
                prefix.push_str("    ");
            }
        }

        // Add the branch connector
        if node.depth > 0 {
            if node.is_last_child {
                prefix.push_str("└── ");
            } else {
                prefix.push_str("├── ");
            }
        }

        prefix
    }
}

/// Get the color for a dependency type
///
/// Returns the appropriate color based on the dependency category:
/// - Production: Green (bundled with the application)
/// - Development: Yellow (only needed during development)
/// - Peer: Cyan (expected to be provided by the consumer)
/// - Optional: Gray (enhance functionality if available)
fn get_dep_type_color(dep_type: Option<DependencyType>) -> Color {
    match dep_type {
        Some(DependencyType::Production) => Color::Green,
        Some(DependencyType::Development) => Color::Yellow,
        Some(DependencyType::Peer) => Color::Cyan,
        Some(DependencyType::Optional) => Color::Gray,
        None => Color::White, // Root node or unknown type
    }
}

/// Get the short type indicator for a dependency type
///
/// Returns a short label for display next to the dependency name:
/// - P: Production
/// - D: Development
/// - Pe: Peer
/// - O: Optional
fn get_dep_type_indicator(dep_type: Option<DependencyType>) -> &'static str {
    match dep_type {
        Some(DependencyType::Production) => "[P] ",
        Some(DependencyType::Development) => "[D] ",
        Some(DependencyType::Peer) => "[Pe] ",
        Some(DependencyType::Optional) => "[O] ",
        None => "", // Root node or unknown type
    }
}

/// Run the TUI application
pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|frame| render(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                    KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
                    KeyCode::Enter | KeyCode::Char(' ') => app.toggle_selected(),
                    _ => {}
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

/// Render the application UI
fn render(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, chunks[0]);
    render_tree(frame, app, chunks[1]);
    render_footer(frame, chunks[2]);
}

/// Render the header
fn render_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new("CodeScope - Dependency Analyzer")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, area);
}

/// Render the dependency tree
pub fn render_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .flattened
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let prefix = app.get_tree_prefix(index);
            let indicator = node.expansion_indicator();
            let dep_color = get_dep_type_color(node.dep_type);
            let type_indicator = get_dep_type_indicator(node.dep_type);

            let content = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(indicator, Style::default().fg(Color::Yellow)),
                Span::styled(type_indicator, Style::default().fg(dep_color)),
                Span::styled(&node.name, Style::default().fg(dep_color)),
                Span::styled(
                    format!(" @{}", node.version),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let tree_block = Block::default()
        .title("Dependencies")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let tree_list = List::new(items)
        .block(tree_block)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    frame.render_stateful_widget(tree_list, area, &mut app.list_state);
}

/// Render the footer with help text and legend
fn render_footer(frame: &mut Frame, area: Rect) {
    let help_text = Line::from(vec![
        Span::styled("j/↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Down  "),
        Span::styled("k/↑", Style::default().fg(Color::Yellow)),
        Span::raw(" Up  "),
        Span::styled("Enter/Space", Style::default().fg(Color::Yellow)),
        Span::raw(" Toggle  "),
        Span::styled("q/Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit  │  "),
        Span::styled("[P]", Style::default().fg(Color::Green)),
        Span::raw(" Prod  "),
        Span::styled("[D]", Style::default().fg(Color::Yellow)),
        Span::raw(" Dev  "),
        Span::styled("[Pe]", Style::default().fg(Color::Cyan)),
        Span::raw(" Peer  "),
        Span::styled("[O]", Style::default().fg(Color::Gray)),
        Span::raw(" Optional"),
    ]);

    let footer = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> App {
        let mut root = TreeNode::new("my-project".to_string(), "1.0.0".to_string());

        let mut dep_a = TreeNode::new("react".to_string(), "18.2.0".to_string());
        dep_a.add_child(TreeNode::new("react-dom".to_string(), "18.2.0".to_string()));

        let dep_b = TreeNode::new("lodash".to_string(), "4.17.21".to_string());

        root.add_child(dep_a);
        root.add_child(dep_b);
        root.expanded = true;

        App::new(root)
    }

    #[test]
    fn test_app_creation() {
        let app = create_test_app();
        assert_eq!(app.selected_index, 0);
        assert!(!app.should_quit);
        // Root is expanded, so we should see root + 2 children
        assert_eq!(app.flattened.len(), 3);
    }

    #[test]
    fn test_select_next() {
        let mut app = create_test_app();
        assert_eq!(app.selected_index, 0);

        app.select_next();
        assert_eq!(app.selected_index, 1);

        app.select_next();
        assert_eq!(app.selected_index, 2);

        // Should not go past the last item
        app.select_next();
        assert_eq!(app.selected_index, 2);
    }

    #[test]
    fn test_select_previous() {
        let mut app = create_test_app();
        app.selected_index = 2;
        app.list_state.select(Some(2));

        app.select_previous();
        assert_eq!(app.selected_index, 1);

        app.select_previous();
        assert_eq!(app.selected_index, 0);

        // Should not go below 0
        app.select_previous();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_toggle_selected() {
        let mut app = create_test_app();

        // Select react (index 1)
        app.selected_index = 1;
        app.list_state.select(Some(1));

        // Toggle to expand
        app.toggle_selected();
        assert_eq!(app.flattened.len(), 4); // Now shows react-dom too

        // Toggle to collapse
        app.toggle_selected();
        assert_eq!(app.flattened.len(), 3);
    }

    #[test]
    fn test_quit() {
        let mut app = create_test_app();
        assert!(!app.should_quit);

        app.quit();
        assert!(app.should_quit);
    }
}
