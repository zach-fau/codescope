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
    /// Filtered flattened view (when search is active)
    pub filtered: Vec<FlattenedNode>,
    /// Track which ancestors are "last child" for proper tree drawing
    ancestors_last: Vec<bool>,
    /// Whether the application should quit
    pub should_quit: bool,
    /// List state for ratatui
    list_state: ListState,
    /// Whether search mode is active
    pub search_active: bool,
    /// Current search query
    pub search_query: String,
}

impl App {
    /// Create a new application with the given root tree node
    pub fn new(root: TreeNode) -> Self {
        let mut app = Self {
            tree: root,
            selected_index: 0,
            flattened: Vec::new(),
            filtered: Vec::new(),
            ancestors_last: Vec::new(),
            should_quit: false,
            list_state: ListState::default(),
            search_active: false,
            search_query: String::new(),
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

    /// Start search mode
    pub fn start_search(&mut self) {
        self.search_active = true;
        self.search_query.clear();
    }

    /// Clear search and return to normal mode
    pub fn clear_search(&mut self) {
        self.search_active = false;
        self.search_query.clear();
        self.filtered.clear();
        // Reset selection to first item
        self.selected_index = 0;
        self.list_state.select(Some(0));
    }

    /// Add a character to the search query
    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filter();
    }

    /// Remove the last character from the search query
    pub fn search_pop(&mut self) {
        self.search_query.pop();
        self.update_filter();
    }

    /// Update the filtered view based on the current search query
    fn update_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered.clear();
            self.selected_index = 0;
        } else {
            self.filtered = self
                .flattened
                .iter()
                .filter(|node| fuzzy_match(&node.name, &self.search_query))
                .cloned()
                .collect();

            // Reset selection if current selection is out of bounds
            if !self.filtered.is_empty() {
                self.selected_index = 0;
            }
        }
        self.list_state.select(Some(self.selected_index));
    }

}

/// Perform fuzzy matching of query against text (case-insensitive)
/// A match requires all characters of the query to appear in order in the text
fn fuzzy_match(text: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    let mut query_chars = query_lower.chars().peekable();
    for c in text_lower.chars() {
        if let Some(&q) = query_chars.peek() {
            if c == q {
                query_chars.next();
            }
        }
        if query_chars.peek().is_none() {
            return true;
        }
    }
    query_chars.peek().is_none()
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
                if app.search_active {
                    // Search mode key handling
                    match key.code {
                        KeyCode::Esc => app.clear_search(),
                        KeyCode::Enter => {
                            // Exit search mode but keep the filter active
                            app.search_active = false;
                        }
                        KeyCode::Backspace => app.search_pop(),
                        KeyCode::Char(c) => app.search_push(c),
                        KeyCode::Down | KeyCode::Tab => app.select_next(),
                        KeyCode::Up | KeyCode::BackTab => app.select_previous(),
                        _ => {}
                    }
                } else {
                    // Normal mode key handling
                    match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Esc => {
                            if !app.search_query.is_empty() {
                                // Clear the filter but stay in normal mode
                                app.clear_search();
                            } else {
                                app.quit();
                            }
                        }
                        KeyCode::Char('/') => app.start_search(),
                        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                        KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
                        KeyCode::Enter | KeyCode::Char(' ') => app.toggle_selected(),
                        _ => {}
                    }
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
    // Determine if search bar is visible
    let show_search = app.search_active || !app.search_query.is_empty();

    let chunks = if show_search {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // Tree
                Constraint::Length(3), // Footer
            ])
            .split(frame.area())
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Tree
                Constraint::Length(3), // Footer
            ])
            .split(frame.area())
    };

    if show_search {
        render_header(frame, chunks[0]);
        render_search_bar(frame, app, chunks[1]);
        render_tree(frame, app, chunks[2]);
        render_footer(frame, app, chunks[3]);
    } else {
        render_header(frame, chunks[0]);
        render_tree(frame, app, chunks[1]);
        render_footer(frame, app, chunks[2]);
    }
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

/// Render the search bar
fn render_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (border_color, title) = if app.search_active {
        (Color::Yellow, "Search (Enter to confirm, Esc to cancel)")
    } else {
        (Color::Gray, "Filter (/ to edit, Esc to clear)")
    };

    let search_display = format!("/{}", app.search_query);
    let cursor = if app.search_active { "_" } else { "" };

    let result_count = if !app.search_query.is_empty() {
        format!(" ({} matches)", app.filtered.len())
    } else {
        String::new()
    };

    let content = Line::from(vec![
        Span::styled(&search_display, Style::default().fg(Color::White)),
        Span::styled(cursor, Style::default().fg(Color::Yellow).add_modifier(Modifier::SLOW_BLINK)),
        Span::styled(&result_count, Style::default().fg(Color::DarkGray)),
    ]);

    let search_bar = Paragraph::new(content)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );
    frame.render_widget(search_bar, area);
}

/// Render the dependency tree
pub fn render_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    // Clone what we need to avoid borrowing issues
    let has_search = !app.search_query.is_empty();
    let search_query = app.search_query.clone();

    // Get the nodes to display - clone to avoid borrow issues
    let display_nodes: Vec<FlattenedNode> = if has_search {
        app.filtered.clone()
    } else {
        app.flattened.clone()
    };

    let items: Vec<ListItem> = display_nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            // Only show tree prefix for non-filtered views
            let prefix = if has_search {
                String::new()
            } else {
                app.get_tree_prefix(index)
            };
            let indicator = node.expansion_indicator();
            let dep_color = get_dep_type_color(node.dep_type);
            let type_indicator = get_dep_type_indicator(node.dep_type);

            // Build the name with highlighting if there's a search query
            let name_spans = if has_search {
                highlight_matches(&node.name, &search_query, dep_color)
            } else {
                vec![Span::styled(node.name.clone(), Style::default().fg(dep_color))]
            };

            let mut content_spans = vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(indicator, Style::default().fg(Color::Yellow)),
                Span::styled(type_indicator, Style::default().fg(dep_color)),
            ];
            content_spans.extend(name_spans);
            content_spans.push(Span::styled(
                format!(" @{}", node.version),
                Style::default().fg(Color::DarkGray),
            ));

            ListItem::new(Line::from(content_spans))
        })
        .collect();

    let title = if has_search {
        format!("Dependencies (filtered: {} matches)", display_nodes.len())
    } else {
        "Dependencies".to_string()
    };

    let tree_block = Block::default()
        .title(title)
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

/// Highlight matching characters in a string based on fuzzy search
fn highlight_matches(text: &str, query: &str, base_color: Color) -> Vec<Span<'static>> {
    if query.is_empty() {
        return vec![Span::styled(text.to_string(), Style::default().fg(base_color))];
    }

    let query_lower = query.to_lowercase();
    let mut result = Vec::new();
    let mut current_segment = String::new();
    let mut current_is_match = false;
    let mut query_chars = query_lower.chars().peekable();

    for c in text.chars() {
        let c_lower = c.to_lowercase().next().unwrap_or(c);
        let is_match = query_chars.peek().is_some_and(|&q| c_lower == q);

        if is_match {
            query_chars.next();
        }

        if is_match != current_is_match && !current_segment.is_empty() {
            // Push the current segment
            let style = if current_is_match {
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(base_color)
            };
            result.push(Span::styled(current_segment.clone(), style));
            current_segment.clear();
        }

        current_segment.push(c);
        current_is_match = is_match;
    }

    // Push the final segment
    if !current_segment.is_empty() {
        let style = if current_is_match {
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(base_color)
        };
        result.push(Span::styled(current_segment, style));
    }

    result
}

/// Render the footer with help text and legend
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.search_active {
        // Search mode help
        Line::from(vec![
            Span::styled("Type", Style::default().fg(Color::Yellow)),
            Span::raw(" to search  "),
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Confirm  "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Cancel"),
        ])
    } else {
        // Normal mode help with search shortcut
        Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw(" Search  "),
            Span::styled("j/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Down  "),
            Span::styled("k/↑", Style::default().fg(Color::Yellow)),
            Span::raw(" Up  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Toggle  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit  │  "),
            Span::styled("[P]", Style::default().fg(Color::Green)),
            Span::raw(" Prod  "),
            Span::styled("[D]", Style::default().fg(Color::Yellow)),
            Span::raw(" Dev  "),
            Span::styled("[Pe]", Style::default().fg(Color::Cyan)),
            Span::raw(" Peer  "),
            Span::styled("[O]", Style::default().fg(Color::Gray)),
            Span::raw(" Opt"),
        ])
    };

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

    #[test]
    fn test_fuzzy_match() {
        // Exact match
        assert!(fuzzy_match("react", "react"));

        // Partial match (substring)
        assert!(fuzzy_match("react", "re"));

        // Fuzzy match (characters in order)
        assert!(fuzzy_match("react", "rct"));

        // Case insensitive
        assert!(fuzzy_match("react", "REACT"));
        assert!(fuzzy_match("React", "react"));

        // No match
        assert!(!fuzzy_match("react", "xyz"));

        // Empty query matches everything
        assert!(fuzzy_match("react", ""));

        // Query longer than text
        assert!(!fuzzy_match("re", "react"));
    }

    #[test]
    fn test_search_start_and_clear() {
        let mut app = create_test_app();

        // Initially not in search mode
        assert!(!app.search_active);
        assert!(app.search_query.is_empty());

        // Start search
        app.start_search();
        assert!(app.search_active);
        assert!(app.search_query.is_empty());

        // Type something
        app.search_push('r');
        app.search_push('e');
        assert_eq!(app.search_query, "re");

        // Clear search
        app.clear_search();
        assert!(!app.search_active);
        assert!(app.search_query.is_empty());
        assert!(app.filtered.is_empty());
    }

    #[test]
    fn test_search_filtering() {
        let mut app = create_test_app();

        // Expand react to add react-dom
        app.selected_index = 1;
        app.list_state.select(Some(1));
        app.toggle_selected();
        assert_eq!(app.flattened.len(), 4); // my-project, react, react-dom, lodash

        // Start search and type "react"
        app.start_search();
        app.search_push('r');
        app.search_push('e');
        app.search_push('a');
        app.search_push('c');
        app.search_push('t');

        // Should match "react" and "react-dom" (both contain "react")
        assert_eq!(app.filtered.len(), 2);
        assert!(app.filtered.iter().any(|n| n.name == "react"));
        assert!(app.filtered.iter().any(|n| n.name == "react-dom"));
    }

    #[test]
    fn test_search_pop() {
        let mut app = create_test_app();
        app.start_search();

        app.search_push('r');
        app.search_push('e');
        assert_eq!(app.search_query, "re");

        app.search_pop();
        assert_eq!(app.search_query, "r");

        app.search_pop();
        assert!(app.search_query.is_empty());

        // Pop on empty doesn't panic
        app.search_pop();
        assert!(app.search_query.is_empty());
    }
}
