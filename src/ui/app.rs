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

/// Virtual scroll state for efficient rendering of large trees
#[derive(Debug, Default, Clone)]
pub struct VirtualScrollState {
    /// First visible row index
    pub offset: usize,
    /// Number of visible rows in the viewport
    pub viewport_height: usize,
}

impl VirtualScrollState {
    /// Create a new virtual scroll state
    pub fn new() -> Self {
        Self {
            offset: 0,
            viewport_height: 0,
        }
    }

    /// Update the viewport height
    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
    }

    /// Calculate the visible range for the given selection and total items
    pub fn visible_range(&self, selected: usize, total: usize) -> (usize, usize) {
        if total == 0 || self.viewport_height == 0 {
            return (0, 0);
        }

        // Ensure selection is visible by adjusting offset
        let mut offset = self.offset;

        // If selection is above visible area, scroll up
        if selected < offset {
            offset = selected;
        }
        // If selection is below visible area, scroll down
        else if selected >= offset + self.viewport_height {
            offset = selected.saturating_sub(self.viewport_height - 1);
        }

        let start = offset;
        let end = (offset + self.viewport_height).min(total);

        (start, end)
    }

    /// Update offset to ensure selection is visible
    pub fn ensure_visible(&mut self, selected: usize, total: usize) {
        if total == 0 || self.viewport_height == 0 {
            return;
        }

        // If selection is above visible area, scroll up
        if selected < self.offset {
            self.offset = selected;
        }
        // If selection is below visible area, scroll down
        else if selected >= self.offset + self.viewport_height {
            self.offset = selected.saturating_sub(self.viewport_height - 1);
        }
    }
}

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
    /// Virtual scroll state for performance with large trees
    pub scroll_state: VirtualScrollState,
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
            scroll_state: VirtualScrollState::new(),
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
        let total = self.current_list_len();
        if total > 0 {
            self.selected_index = (self.selected_index + 1).min(total - 1);
            self.list_state.select(Some(self.selected_index));
            self.scroll_state.ensure_visible(self.selected_index, total);
        }
    }

    /// Move selection to the previous item
    pub fn select_previous(&mut self) {
        let total = self.current_list_len();
        if total > 0 && self.selected_index > 0 {
            self.selected_index -= 1;
            self.list_state.select(Some(self.selected_index));
            self.scroll_state.ensure_visible(self.selected_index, total);
        }
    }

    /// Move selection down by a page
    pub fn page_down(&mut self) {
        let total = self.current_list_len();
        if total > 0 {
            let page_size = self.scroll_state.viewport_height.max(1);
            self.selected_index = (self.selected_index + page_size).min(total - 1);
            self.list_state.select(Some(self.selected_index));
            self.scroll_state.ensure_visible(self.selected_index, total);
        }
    }

    /// Move selection up by a page
    pub fn page_up(&mut self) {
        let total = self.current_list_len();
        if total > 0 {
            let page_size = self.scroll_state.viewport_height.max(1);
            self.selected_index = self.selected_index.saturating_sub(page_size);
            self.list_state.select(Some(self.selected_index));
            self.scroll_state.ensure_visible(self.selected_index, total);
        }
    }

    /// Jump to the first item
    pub fn select_first(&mut self) {
        let total = self.current_list_len();
        if total > 0 {
            self.selected_index = 0;
            self.list_state.select(Some(0));
            self.scroll_state.offset = 0;
        }
    }

    /// Jump to the last item
    pub fn select_last(&mut self) {
        let total = self.current_list_len();
        if total > 0 {
            self.selected_index = total - 1;
            self.list_state.select(Some(self.selected_index));
            self.scroll_state.ensure_visible(self.selected_index, total);
        }
    }

    /// Get the current list length (filtered or full)
    fn current_list_len(&self) -> usize {
        if !self.search_query.is_empty() {
            self.filtered.len()
        } else {
            self.flattened.len()
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

/// Get the color for a dependency type, with cycle and conflict overrides
///
/// Returns the appropriate color based on the dependency category:
/// - Cycle nodes: Red (circular dependency warning - highest priority)
/// - Conflict nodes: Rgb(255, 165, 0) orange (version conflict warning)
/// - Production: Green (bundled with the application)
/// - Development: Yellow (only needed during development)
/// - Peer: Cyan (expected to be provided by the consumer)
/// - Optional: Gray (enhance functionality if available)
fn get_dep_type_color(dep_type: Option<DependencyType>, is_in_cycle: bool, has_conflict: bool) -> Color {
    // Cycle nodes are always shown in red regardless of dependency type (highest priority)
    if is_in_cycle {
        return Color::Red;
    }
    // Conflict nodes shown in orange
    if has_conflict {
        return Color::Rgb(255, 165, 0); // Orange color
    }
    match dep_type {
        Some(DependencyType::Production) => Color::Green,
        Some(DependencyType::Development) => Color::Yellow,
        Some(DependencyType::Peer) => Color::Cyan,
        Some(DependencyType::Optional) => Color::Gray,
        None => Color::White, // Root node or unknown type
    }
}

/// Maximum depth for color gradient calculations
const MAX_DEPTH_FOR_COLOR: usize = 10;

/// Get color intensity based on depth (brighter = closer to root)
///
/// Returns a brightness factor from 0.0 to 1.0 where:
/// - Depth 0 (root): 1.0 (brightest)
/// - Max depth: 0.4 (dimmer but still visible)
fn get_depth_brightness(depth: usize) -> f32 {
    let clamped_depth = depth.min(MAX_DEPTH_FOR_COLOR);
    let ratio = clamped_depth as f32 / MAX_DEPTH_FOR_COLOR as f32;
    // Linear interpolation from 1.0 (bright) to 0.4 (dim)
    1.0 - (ratio * 0.6)
}

/// Apply brightness modifier to a color based on depth
///
/// Adjusts the color brightness so deeper nodes appear dimmer,
/// making the dependency chain depth immediately visible.
fn apply_depth_color(base_color: Color, depth: usize) -> Color {
    let brightness = get_depth_brightness(depth);

    match base_color {
        Color::Rgb(r, g, b) => {
            Color::Rgb(
                (r as f32 * brightness) as u8,
                (g as f32 * brightness) as u8,
                (b as f32 * brightness) as u8,
            )
        }
        Color::Green => {
            let base = 255_f32;
            Color::Rgb(0, (base * brightness) as u8, 0)
        }
        Color::Yellow => {
            let base = 255_f32;
            Color::Rgb((base * brightness) as u8, (base * brightness) as u8, 0)
        }
        Color::Cyan => {
            let base = 255_f32;
            Color::Rgb(0, (base * brightness) as u8, (base * brightness) as u8)
        }
        Color::Red => {
            let base = 255_f32;
            Color::Rgb((base * brightness) as u8, 0, 0)
        }
        Color::Gray => {
            let base = 128_f32;
            let val = (base * brightness) as u8;
            Color::Rgb(val, val, val)
        }
        Color::White => {
            let base = 255_f32;
            let val = (base * brightness) as u8;
            Color::Rgb(val, val, val)
        }
        // For other color types, return as-is
        other => other,
    }
}

/// Get the depth indicator string for a node
///
/// Returns a depth level indicator that shows how deep in the
/// dependency tree this node is located.
fn get_depth_indicator(depth: usize) -> String {
    if depth == 0 {
        String::new() // Root node doesn't need depth indicator
    } else {
        format!("L{} ", depth)
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

/// Get the cycle indicator if the node is part of a circular dependency
///
/// Returns a warning symbol for nodes in cycles
fn get_cycle_indicator(is_in_cycle: bool) -> &'static str {
    if is_in_cycle {
        "[!] "
    } else {
        ""
    }
}

/// Get the conflict indicator if the node has version conflicts
///
/// Returns a warning symbol for nodes with conflicts
fn get_conflict_indicator(has_conflict: bool) -> &'static str {
    if has_conflict {
        "[~] "
    } else {
        ""
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
                        // Page navigation for large trees
                        KeyCode::PageDown | KeyCode::Char('d') => app.page_down(),
                        KeyCode::PageUp | KeyCode::Char('u') => app.page_up(),
                        KeyCode::Home | KeyCode::Char('g') => app.select_first(),
                        KeyCode::End | KeyCode::Char('G') => app.select_last(),
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

/// Render the dependency tree with virtual scrolling
///
/// Only renders visible nodes for performance with large trees (1000+ nodes).
/// Updates the scroll state viewport height based on available area.
pub fn render_tree(frame: &mut Frame, app: &mut App, area: Rect) {
    // Clone what we need to avoid borrowing issues
    let has_search = !app.search_query.is_empty();
    let search_query = app.search_query.clone();

    // Get the nodes to display
    let display_nodes: &[FlattenedNode] = if has_search {
        &app.filtered
    } else {
        &app.flattened
    };

    let total_nodes = display_nodes.len();

    // Calculate viewport height (area height minus borders)
    // Border takes 2 rows (top + bottom)
    let viewport_height = (area.height as usize).saturating_sub(2);
    app.scroll_state.set_viewport_height(viewport_height);

    // Ensure selection is visible and get visible range
    app.scroll_state.ensure_visible(app.selected_index, total_nodes);
    let (start_idx, end_idx) = app.scroll_state.visible_range(app.selected_index, total_nodes);

    // Only render visible nodes (virtual scrolling optimization)
    let visible_nodes = &display_nodes[start_idx..end_idx];

    let items: Vec<ListItem> = visible_nodes
        .iter()
        .enumerate()
        .map(|(visible_idx, node)| {
            // Calculate actual index in the full list
            let actual_index = start_idx + visible_idx;

            // Only show tree prefix for non-filtered views
            let prefix = if has_search {
                String::new()
            } else {
                app.get_tree_prefix(actual_index)
            };
            let indicator = node.expansion_indicator();
            let base_dep_color = get_dep_type_color(node.dep_type, node.is_in_cycle, node.has_conflict);
            // Apply depth-based color gradient (brighter = closer to root)
            let dep_color = apply_depth_color(base_dep_color, node.depth);
            let type_indicator = get_dep_type_indicator(node.dep_type);
            let cycle_indicator = get_cycle_indicator(node.is_in_cycle);
            let conflict_indicator = get_conflict_indicator(node.has_conflict);
            let depth_indicator = get_depth_indicator(node.depth);

            // Build the name with highlighting if there's a search query
            let name_spans = if has_search {
                highlight_matches(&node.name, &search_query, dep_color)
            } else {
                vec![Span::styled(node.name.clone(), Style::default().fg(dep_color))]
            };

            // Depth indicator color - blue gradient based on depth
            let depth_color = apply_depth_color(Color::Rgb(100, 149, 237), node.depth); // Cornflower blue

            let mut content_spans = vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::styled(indicator, Style::default().fg(Color::Yellow)),
                Span::styled(depth_indicator, Style::default().fg(depth_color)),
                Span::styled(cycle_indicator, Style::default().fg(Color::Red)),
                Span::styled(conflict_indicator, Style::default().fg(Color::Rgb(255, 165, 0))),
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

    // Adjust list_state selection to be relative to visible window
    let relative_selection = app.selected_index.saturating_sub(start_idx);
    app.list_state.select(Some(relative_selection));

    // Build title with scroll position indicator for large trees
    let title = if has_search {
        format!("Dependencies (filtered: {} matches)", total_nodes)
    } else if total_nodes > viewport_height {
        // Show scroll position for large trees
        format!(
            "Dependencies ({}-{} of {})",
            start_idx + 1,
            end_idx,
            total_nodes
        )
    } else {
        format!("Dependencies ({})", total_nodes)
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
        // Normal mode help with search shortcut and page navigation
        Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw(" Search  "),
            Span::styled("j/k", Style::default().fg(Color::Yellow)),
            Span::raw(" Nav  "),
            Span::styled("d/u", Style::default().fg(Color::Yellow)),
            Span::raw(" Page  "),
            Span::styled("g/G", Style::default().fg(Color::Yellow)),
            Span::raw(" Top/Bot  "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(" Toggle  "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit  │  "),
            Span::styled("[P]", Style::default().fg(Color::Green)),
            Span::raw(" Prod  "),
            Span::styled("[D]", Style::default().fg(Color::Yellow)),
            Span::raw(" Dev  "),
            Span::styled("[!]", Style::default().fg(Color::Red)),
            Span::raw(" Cycle  "),
            Span::styled("L#", Style::default().fg(Color::Rgb(100, 149, 237))),
            Span::raw(" Depth"),
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

    #[test]
    fn test_depth_brightness() {
        // Root node should be brightest
        assert!((get_depth_brightness(0) - 1.0).abs() < f32::EPSILON);

        // Middle depth
        let mid_brightness = get_depth_brightness(5);
        assert!(mid_brightness > 0.4);
        assert!(mid_brightness < 1.0);

        // Max depth should be dimmest
        let max_brightness = get_depth_brightness(MAX_DEPTH_FOR_COLOR);
        assert!((max_brightness - 0.4).abs() < f32::EPSILON);

        // Beyond max depth should clamp to dimmest
        let beyond_max = get_depth_brightness(MAX_DEPTH_FOR_COLOR + 5);
        assert!((beyond_max - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn test_apply_depth_color() {
        // Root (depth 0) should preserve full brightness
        let root_color = apply_depth_color(Color::Green, 0);
        assert_eq!(root_color, Color::Rgb(0, 255, 0));

        // Deeper nodes should be dimmer
        let deep_color = apply_depth_color(Color::Green, MAX_DEPTH_FOR_COLOR);
        match deep_color {
            Color::Rgb(_, g, _) => {
                assert!(g < 255);
                assert!(g > 50); // Still visible
            }
            _ => panic!("Expected RGB color"),
        }
    }

    #[test]
    fn test_depth_indicator() {
        // Root node has no depth indicator
        assert_eq!(get_depth_indicator(0), "");

        // Other depths show level
        assert_eq!(get_depth_indicator(1), "L1 ");
        assert_eq!(get_depth_indicator(5), "L5 ");
        assert_eq!(get_depth_indicator(10), "L10 ");
    }

    #[test]
    fn test_virtual_scroll_state_new() {
        let state = VirtualScrollState::new();
        assert_eq!(state.offset, 0);
        assert_eq!(state.viewport_height, 0);
    }

    #[test]
    fn test_virtual_scroll_visible_range() {
        let mut state = VirtualScrollState::new();
        state.set_viewport_height(10);

        // At start
        let (start, end) = state.visible_range(0, 100);
        assert_eq!(start, 0);
        assert_eq!(end, 10);

        // Selection in middle (offset should adjust)
        state.offset = 0;
        let (start, end) = state.visible_range(50, 100);
        assert!(start <= 50);
        assert!(end > 50);

        // At end
        let (start, end) = state.visible_range(99, 100);
        assert!(start >= 90);
        assert_eq!(end, 100);

        // Empty list
        let (start, end) = state.visible_range(0, 0);
        assert_eq!(start, 0);
        assert_eq!(end, 0);
    }

    #[test]
    fn test_virtual_scroll_ensure_visible() {
        let mut state = VirtualScrollState::new();
        state.set_viewport_height(10);
        state.offset = 50;

        // Selection above visible area - should scroll up
        state.ensure_visible(40, 100);
        assert_eq!(state.offset, 40);

        // Selection below visible area - should scroll down
        state.offset = 0;
        state.ensure_visible(15, 100);
        assert!(state.offset > 0);
        assert!(state.offset + state.viewport_height > 15);
    }

    #[test]
    fn test_page_navigation() {
        let mut app = create_test_app();
        app.scroll_state.set_viewport_height(2);

        // Expand everything
        app.tree.expanded = true;
        for child in &mut app.tree.children {
            child.expanded = true;
        }
        app.refresh_flattened();

        // Page down
        app.selected_index = 0;
        app.page_down();
        assert_eq!(app.selected_index, 2);

        // Page up
        app.page_up();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_select_first_last() {
        let mut app = create_test_app();

        // Go to last
        app.select_last();
        assert_eq!(app.selected_index, app.flattened.len() - 1);

        // Go to first
        app.select_first();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_current_list_len() {
        let mut app = create_test_app();

        // Without search
        assert_eq!(app.current_list_len(), app.flattened.len());

        // With search
        app.start_search();
        app.search_push('r');
        assert_eq!(app.current_list_len(), app.filtered.len());
    }
}
