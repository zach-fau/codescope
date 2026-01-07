# CodeScope Architecture

## Overview

CodeScope is a terminal UI dependency analyzer built in Rust. It analyzes project dependencies, visualizes the dependency tree, and calculates bundle size impact.

## Design Principles

1. **Performance First**: Handle 1000+ dependencies in < 2 seconds
2. **Terminal Native**: Fits developer CLI workflows
3. **Modular**: Easy to add new language support
4. **Cross-Platform**: Works on Linux, macOS, Windows

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│                        (main.rs, cli/)                       │
└────────────────┬─────────────────────────┬──────────────────┘
                 │                         │
        ┌────────▼──────────┐    ┌────────▼──────────┐
        │   Parser Layer     │    │    UI Layer       │
        │   (parser/)        │    │    (ui/)          │
        │                    │    │                    │
        │  - package.json    │    │  - Tree Widget    │
        │  - Cargo.toml      │    │  - Keyboard Nav   │
        │  - go.mod          │    │  - Color Scheme   │
        │  - Bundle Stats    │    │  - Status Bar     │
        └────────┬───────────┘    └────────┬──────────┘
                 │                         │
                 │  ┌─────────────────────▼────────┐
                 └─►│      Graph Layer             │
                    │      (graph/)                │
                    │                              │
                    │  - Dependency Graph          │
                    │  - Cycle Detection           │
                    │  - Size Attribution          │
                    │  - Export Analysis           │
                    └──────────────────────────────┘
```

## Component Details

### 1. CLI Layer (`main.rs`, `cli/`)

**Responsibility**: Command-line interface and user interaction

**Key Components**:
- `main.rs`: Entry point, argument parsing (clap)
- `cli/commands.rs`: Command implementations
- `cli/config.rs`: Configuration file handling

**Dependencies**:
- `clap` for argument parsing
- `anyhow` for error handling

### 2. Parser Layer (`parser/`)

**Responsibility**: Parse dependency manifests and build output

**Key Modules**:
- `parser/package_json.rs`: Parse package.json
- `parser/cargo.rs`: Parse Cargo.toml (future)
- `parser/go_mod.rs`: Parse go.mod (future)
- `parser/bundle_stats.rs`: Parse webpack/vite output

**Data Flow**:
```
File → Read → Parse → DependencyManifest → Graph
```

**Error Handling**:
- Invalid JSON → user-friendly error
- Missing file → clear instructions
- Malformed data → partial parsing with warnings

### 3. Graph Layer (`graph/`)

**Responsibility**: Build and analyze dependency graph

**Key Structures**:
```rust
pub struct DependencyGraph {
    graph: DiGraph<Package, DependencyRelation>,
    metadata: GraphMetadata,
}

pub struct Package {
    name: String,
    version: String,
    package_type: PackageType,
    size: Option<usize>,
}

pub enum DependencyRelation {
    Direct,
    Dev,
    Peer,
    Optional,
}
```

**Algorithms**:
- **Cycle Detection**: Tarjan's algorithm (O(V+E))
- **Size Attribution**: DFS traversal with caching
- **Unused Exports**: AST analysis with tree-sitter

### 4. UI Layer (`ui/`)

**Responsibility**: Terminal user interface

**Key Components**:
- `ui/tree.rs`: Collapsible tree widget
- `ui/theme.rs`: Color scheme and styling
- `ui/keyboard.rs`: Keyboard event handling
- `ui/layout.rs`: Screen layout management

**Rendering Pipeline**:
```
Graph Data → Tree State → ratatui Widgets → Terminal
```

**Interaction Model**:
- Vim-like navigation (hjkl, /, gg, G)
- Expandable/collapsible nodes
- Search and filter
- Real-time updates

## Data Structures

### DependencyManifest
```rust
pub struct DependencyManifest {
    pub name: String,
    pub version: String,
    pub dependencies: HashMap<String, Dependency>,
    pub dev_dependencies: HashMap<String, Dependency>,
    pub peer_dependencies: HashMap<String, Dependency>,
}
```

### BundleStats
```rust
pub struct BundleStats {
    pub total_size: usize,
    pub modules: Vec<ModuleInfo>,
    pub assets: Vec<AssetInfo>,
}

pub struct ModuleInfo {
    pub path: String,
    pub size: usize,
    pub chunks: Vec<String>,
}
```

## Performance Considerations

### Parsing
- **Lazy Loading**: Parse files on-demand
- **Streaming**: Process large files incrementally
- **Caching**: Cache parsed manifests

### Graph Operations
- **Sparse Representation**: Use petgraph's DiGraph
- **Memoization**: Cache size calculations
- **Parallel**: Use rayon for independent subtrees (future)

### UI Rendering
- **Viewport Culling**: Only render visible nodes
- **Double Buffering**: ratatui handles this
- **Incremental Updates**: Only redraw changed regions

## Testing Strategy

### Unit Tests
- Parser correctness
- Graph algorithms
- Size calculations

### Integration Tests
- End-to-end CLI workflows
- Real-world project parsing
- Error handling

### Performance Tests
- Benchmark with criterion
- Large dependency trees (1000+ nodes)
- Memory usage profiling

## Build Optimization

### Release Profile
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Better optimization, slower builds
strip = true           # Remove debug symbols
```

### Binary Size
- Current: ~5MB (with dependencies)
- Target: < 10MB
- Techniques: LTO, strip, minimal dependencies

## Future Architecture Changes

### Phase 2: Multi-Language Support
```
parser/
├── javascript/
│   ├── package_json.rs
│   └── bundle_stats.rs
├── rust/
│   └── cargo.rs
├── go/
│   └── go_mod.rs
└── python/
    └── pyproject.rs
```

### Phase 3: Watch Mode
```rust
pub struct Watcher {
    watcher: RecommendedWatcher,
    tx: Sender<Event>,
    rx: Receiver<Event>,
}
```

### Phase 4: Plugin System
```rust
pub trait LanguagePlugin {
    fn parse_manifest(&self, path: &Path) -> Result<DependencyManifest>;
    fn analyze_bundle(&self, path: &Path) -> Result<BundleStats>;
}
```

## Security Considerations

1. **File Access**: Only read files in specified directories
2. **Command Injection**: No shell command execution
3. **Dependency Auditing**: Regular `cargo audit`
4. **User Input**: Sanitize file paths and user input

## Error Handling

### Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum CodescopeError {
    #[error("Failed to parse {file}: {source}")]
    ParseError {
        file: PathBuf,
        source: serde_json::Error,
    },

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
}
```

### Error Recovery
- Graceful degradation for missing data
- Partial results when possible
- Clear error messages with suggested fixes

## Logging and Diagnostics

```rust
// Development
RUST_LOG=debug cargo run -- analyze

// Production
codescope analyze --verbose
```

## Cross-Platform Support

### Platform-Specific Code
```rust
#[cfg(target_os = "windows")]
fn get_path_separator() -> char { '\\' }

#[cfg(not(target_os = "windows"))]
fn get_path_separator() -> char { '/' }
```

### Terminal Compatibility
- ANSI color support detection
- Fallback to monochrome
- Unicode handling

---

**Last Updated**: January 7, 2026
**Status**: Week 1 - Foundation Phase
