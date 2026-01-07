# CodeScope Architecture

> Terminal UI dependency analyzer with bundle size impact visualization

## Overview

CodeScope is a Rust-based command-line tool that analyzes JavaScript/TypeScript projects to visualize their dependency trees and bundle size impact. It combines multiple analysis techniques to give developers actionable insights into their dependency graph and bundle optimization opportunities.

### Architecture Philosophy

1. **Separation of Concerns** - Each module handles one responsibility
2. **Data-Driven Design** - Parse once, transform through pipeline
3. **Performance First** - Zero-copy parsing where possible, efficient graph algorithms
4. **User-Focused Output** - Both interactive TUI and scriptable CLI output

## Module Structure

```
codescope/
├── src/
│   ├── main.rs               # CLI entry point, orchestration
│   ├── lib.rs                # Library exports, public API
│   │
│   ├── parser/               # Input parsing layer
│   │   ├── mod.rs            # Module exports
│   │   ├── types.rs          # Core data types (PackageJson, Dependency)
│   │   └── package_json.rs   # package.json parsing logic
│   │
│   ├── graph/                # Dependency graph layer
│   │   ├── mod.rs            # Module exports
│   │   └── dependency_graph.rs  # Graph construction & traversal
│   │
│   ├── analysis/             # Source code analysis
│   │   ├── mod.rs            # Module exports
│   │   └── exports.rs        # Import/export tracking (tree-sitter)
│   │
│   ├── bundle/               # Bundle size analysis
│   │   ├── mod.rs            # Module exports
│   │   ├── webpack.rs        # Webpack stats.json parsing
│   │   └── savings.rs        # Size savings calculation
│   │
│   └── ui/                   # Terminal UI layer
│       ├── mod.rs            # Module exports
│       ├── app.rs            # Application state & event loop
│       └── tree.rs           # Tree rendering (flatten/expand)
│
├── test-project/             # Sample project for testing
└── Cargo.toml                # Dependencies & build config
```

### Module Relationships

```
                    ┌─────────────────┐
                    │     main.rs     │
                    │  (CLI, clap)    │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
       ┌──────────┐   ┌──────────┐   ┌──────────┐
       │  parser/ │   │ analysis/│   │  bundle/ │
       │          │   │          │   │          │
       │ pkg.json │   │tree-sitter│  │ webpack  │
       └────┬─────┘   └────┬─────┘   └────┬─────┘
            │              │              │
            └──────────────┼──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │    graph/    │
                    │  (petgraph)  │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │     ui/      │
                    │  (ratatui)   │
                    └──────────────┘
```

## Data Flow

### Primary Data Pipeline

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  package.json   │───▶│   PackageJson   │───▶│ DependencyGraph │
│  (file on disk) │    │    (struct)     │    │   (petgraph)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                      │
                                                      ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ stats.json      │───▶│  WebpackStats   │───▶│ BundleAnalysis  │
│ (webpack build) │    │    (struct)     │    │ (per-pkg sizes) │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                      │
                                                      ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Source Files   │───▶│ ProjectImports  │───▶│ SavingsReport   │
│  (*.ts, *.js)   │    │ (tree-sitter)   │    │ (optimization)  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                      │
                                                      ▼
                                              ┌─────────────────┐
                                              │   TUI / JSON    │
                                              │   (output)      │
                                              └─────────────────┘
```

### Execution Flow

1. **CLI Parsing** (`main.rs`)
   - Parse command-line arguments with `clap`
   - Validate paths and options

2. **Data Collection** (parallel where possible)
   - Parse `package.json` for declared dependencies
   - Parse `stats.json` for actual bundle sizes (optional)
   - Analyze source files for import usage (optional)

3. **Graph Construction** (`graph/`)
   - Build directed graph with petgraph
   - Nodes = packages, Edges = dependency relationships

4. **Enrichment**
   - Annotate graph nodes with bundle sizes
   - Calculate utilization percentages
   - Identify optimization opportunities

5. **Output**
   - TUI mode: Interactive tree explorer
   - No-TUI mode: JSON/text output for CI/scripts

## Key Components

### Parser Module

**Purpose**: Parse and validate `package.json` files.

**Core Types**:

```rust
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, String>,
    pub dev_dependencies: HashMap<String, String>,
    pub peer_dependencies: HashMap<String, String>,
}

pub struct Dependency {
    pub name: String,
    pub version: String,
    pub dep_type: DependencyType,  // Production, Dev, Peer
}
```

**Key Functions**:
- `PackageJson::from_file(path)` - Parse JSON file with serde
- `PackageJson::all_dependencies()` - Merge all dependency types

### Graph Module

**Purpose**: Build and traverse the dependency graph using petgraph.

**Graph Structure**:
```rust
pub type DependencyGraph = DiGraph<DependencyNode, DependencyEdge>;

pub struct DependencyNode {
    pub name: String,
    pub version: String,
    pub dep_type: DependencyType,
    pub bundle_size: Option<u64>,     // From webpack stats
    pub utilization: Option<f64>,     // From import analysis
    pub potential_savings: Option<u64>,
}

pub struct DependencyEdge {
    pub dep_type: DependencyType,
}
```

**Why petgraph?**
- Battle-tested graph algorithms (DFS, BFS, topological sort)
- Memory-efficient representation
- Flexible node/edge data
- No unsafe code required

**Key Operations**:
- `build_graph(package_json)` - Construct graph from dependencies
- `find_dependents(node)` - Reverse lookup: who depends on this?
- `calculate_depths()` - Compute tree depth for each node

### TUI Module

**Purpose**: Interactive terminal UI using ratatui.

**App State**:
```rust
pub struct App {
    pub tree: Vec<TreeNode>,           // Full tree structure
    pub flattened: Vec<FlattenedNode>, // Visible nodes (expanded)
    pub selected: usize,               // Cursor position
    pub scroll_offset: usize,          // Viewport scroll
    pub sort_by: SortField,            // Current sort
}
```

**Tree Representation**:
```rust
pub struct TreeNode {
    pub dependency: Dependency,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
    pub bundle_size: Option<u64>,
}

pub struct FlattenedNode {
    pub node: TreeNode,
    pub depth: usize,              // For indentation
    pub is_last: bool,             // For drawing └── vs ├──
    pub parent_last_flags: Vec<bool>, // For vertical lines
}
```

**Event Loop Pattern**:
```rust
loop {
    terminal.draw(|f| ui::render(f, &app))?;

    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                KeyCode::Enter | KeyCode::Char(' ') => app.toggle_expand(),
                KeyCode::Char('s') => app.cycle_sort(),
                _ => {}
            }
        }
    }
}
```

**Why ratatui?**
- Modern successor to tui-rs
- Immediate-mode rendering (no retained state)
- Cross-platform terminal handling via crossterm
- Rich widget ecosystem

### Bundle Size Analysis

**Purpose**: Parse webpack stats and calculate per-package bundle impact.

**Webpack Stats Structure**:
```rust
pub struct WebpackStats {
    pub modules: Vec<WebpackModule>,
    pub assets: Vec<WebpackAsset>,
    pub chunks: Vec<WebpackChunk>,
}

pub struct WebpackModule {
    pub name: Option<String>,      // e.g., "./node_modules/lodash/index.js"
    pub size: u64,                 // Size in bytes
    pub chunks: Vec<ChunkId>,
}
```

**Package Name Extraction**:
```rust
// "./node_modules/lodash/lodash.js" -> "lodash"
// "./node_modules/@babel/core/lib/index.js" -> "@babel/core"
pub fn extract_package_name(module_path: &str) -> Option<String>
```

**Key Aggregations**:
- `BundleAnalysis.package_sizes` - Total size per npm package
- `BundleAnalysis.packages_by_size()` - Sorted by bundle impact

### Source Analysis (Tree-sitter)

**Purpose**: Parse JS/TS source files to track import usage.

**Import Types Tracked**:
```rust
pub enum ImportSpecifier {
    Default(String),              // import foo from 'pkg'
    Named { imported, local },    // import { foo as bar } from 'pkg'
    Namespace(String),            // import * as pkg from 'pkg'
    SideEffect,                   // import 'pkg'
    Entire(String),               // const pkg = require('pkg')
}
```

**Package Usage Tracking**:
```rust
pub struct PackageUsage {
    pub named_imports: HashSet<String>,  // Which exports are used
    pub uses_default: bool,
    pub uses_namespace: bool,
    pub has_side_effects: bool,
    pub importing_files: HashSet<String>, // Which files use this
}
```

**Utilization Calculation**:
```rust
// If lodash has 300 exports and you use 2:
// utilization = 2/300 = 0.67%
pub fn utilization_percentage(&self, total_exports: usize) -> Option<f64>
```

**Why Tree-sitter?**
- Incremental parsing (fast on large files)
- Concrete syntax tree (preserves all source details)
- Language-agnostic query system
- Battle-tested in major editors (VS Code, Neovim)

### Savings Calculation

**Purpose**: Identify bundle optimization opportunities.

**Savings Categories**:
```rust
pub enum SavingsCategory {
    Unused,           // Package in deps but not imported
    Underutilized,    // < 20% of exports used
    TreeShaking,      // Moderate usage, tree-shake opportunity
    HasAlternative,   // Known lighter alternative exists
}
```

**Known Alternatives Database**:
```rust
// Heavy package -> (lighter alternative, explanation)
"moment" -> ("dayjs", "Day.js is 2KB vs Moment's 67KB")
"lodash" -> ("lodash-es", "Use lodash-es for tree-shaking")
"axios"  -> ("fetch", "Native fetch is zero-cost")
```

## Design Decisions

### Why Rust?

1. **Performance**: Zero-cost abstractions, no GC pauses
2. **Reliability**: Compiler catches bugs at compile time
3. **Cross-platform**: Single binary, works on macOS/Linux/Windows
4. **Ecosystem**: Excellent CLI/TUI libraries (clap, ratatui)
5. **Safety**: Memory safety without runtime overhead

### Why ratatui over alternatives?

| Alternative | Why Not |
|-------------|---------|
| ncurses | C bindings, platform-specific |
| termion | Less active, fewer widgets |
| tui-rs | Deprecated, ratatui is the successor |
| Ink/Blessed | Requires Node.js runtime |

**ratatui advantages**:
- Pure Rust, cross-platform
- Immediate-mode (simple mental model)
- Active development, good docs
- Works with crossterm for portability

### Why petgraph for the dependency graph?

| Alternative | Why Not |
|-------------|---------|
| Custom adjacency list | Reinventing the wheel |
| daggy | Less flexible, fewer algorithms |
| HashMap<String, Vec<String>> | No traversal algorithms |

**petgraph advantages**:
- Standard Rust graph library
- Efficient memory layout
- Rich algorithm library
- Supports both directed and undirected graphs

### Why tree-sitter for source analysis?

| Alternative | Why Not |
|-------------|---------|
| Regex | Can't handle nested structures |
| swc/oxc | Heavier, full compiler features |
| nom | Would need to write parser from scratch |

**tree-sitter advantages**:
- Production-ready JS/TS grammars
- Handles edge cases (JSX, decorators, etc.)
- Incremental parsing for watch mode
- Same parser used in real editors

## Performance Considerations

### Large Codebases

**Challenge**: Node.js projects can have thousands of dependencies.

**Solutions**:
1. **Lazy Loading**: Only parse transitive deps on expand
2. **Virtualized Rendering**: TUI only renders visible rows
3. **Indexed Lookups**: HashMap for O(1) package lookup
4. **Streaming Parsing**: Process webpack stats incrementally

### Memory Efficiency

```rust
// Use references where possible
pub struct FlattenedNode<'a> {
    pub node: &'a TreeNode,  // Reference, not clone
    // ...
}

// Intern common strings
pub struct StringInterner {
    strings: HashSet<String>,
}
```

### Rendering Performance

```rust
// Only re-flatten when tree structure changes
impl App {
    fn toggle_expand(&mut self) {
        self.tree[self.selected].expanded ^= true;
        self.refresh_flattened();  // Rebuild only when needed
    }
}
```

## Extension Points

### Adding New Package Managers

To support `Cargo.toml`, `go.mod`, etc.:

1. Create new parser in `parser/`:
   ```rust
   // src/parser/cargo_toml.rs
   pub struct CargoToml { /* ... */ }
   impl CargoToml {
       pub fn from_file(path: &Path) -> Result<Self>
   }
   ```

2. Implement a trait for unified interface:
   ```rust
   pub trait ManifestParser {
       fn dependencies(&self) -> Vec<Dependency>;
       fn dev_dependencies(&self) -> Vec<Dependency>;
   }
   ```

3. Add CLI flag for package manager detection:
   ```rust
   #[arg(long, value_enum)]
   package_manager: Option<PackageManager>,
   ```

### Adding New Export Formats

To add CSV, HTML, or other output formats:

1. Define output trait:
   ```rust
   pub trait OutputFormat {
       fn render(&self, graph: &DependencyGraph) -> String;
   }
   ```

2. Implement for each format:
   ```rust
   pub struct CsvOutput;
   impl OutputFormat for CsvOutput {
       fn render(&self, graph: &DependencyGraph) -> String {
           // Generate CSV
       }
   }
   ```

3. Add CLI flag:
   ```rust
   #[arg(long, value_enum, default_value = "json")]
   format: OutputFormat,
   ```

### Adding New Analysis Features

To add security auditing, license checking, etc.:

1. Create new analysis module:
   ```rust
   // src/analysis/security.rs
   pub struct SecurityAudit {
       pub vulnerabilities: Vec<Vulnerability>,
   }
   ```

2. Integrate with graph enrichment:
   ```rust
   pub struct DependencyNode {
       // ... existing fields
       pub vulnerabilities: Vec<Vulnerability>,
   }
   ```

3. Add TUI visualization:
   ```rust
   // Red highlight for vulnerable packages
   if node.vulnerabilities.len() > 0 {
       style = style.fg(Color::Red);
   }
   ```

## Testing Strategy

### Unit Tests

Each module has inline tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_name() {
        assert_eq!(
            extract_package_name("./node_modules/lodash/index.js"),
            Some("lodash".to_string())
        );
    }
}
```

### Integration Tests

End-to-end tests with `test-project/`:

```rust
#[test]
fn test_full_analysis_pipeline() {
    let pkg = PackageJson::from_file("test-project/package.json")?;
    let graph = build_graph(&pkg)?;
    assert!(graph.node_count() > 0);
}
```

### Benchmarks

Performance benchmarks with criterion:

```rust
fn bench_tree_flatten(c: &mut Criterion) {
    let tree = create_large_tree(1000);
    c.bench_function("flatten_tree", |b| {
        b.iter(|| flatten_tree(&tree))
    });
}
```

## Future Directions

1. **Watch Mode** - Re-analyze on file changes (tokio feature flag)
2. **Remote Registry Lookup** - Fetch package metadata from npm
3. **Diff Mode** - Compare two package.json versions
4. **Export to Visualization Tools** - D3.js, Mermaid diagrams
5. **Language Server Protocol** - Editor integration

---

*Last updated: January 2026*
