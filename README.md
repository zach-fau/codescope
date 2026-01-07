# CodeScope

**Terminal UI dependency analyzer with bundle size impact visualization**

[![Build Status](https://img.shields.io/github/actions/workflow/status/zach-fau/codescope/ci.yml?branch=main)](https://github.com/zach-fau/codescope/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/codescope.svg)](https://crates.io/crates/codescope)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

---

CodeScope is a fast, terminal-native tool for analyzing JavaScript/TypeScript project dependencies and understanding their real impact on your bundle size. Built in Rust for performance and cross-platform support.

```
codescope analyze

CodeScope - Dependency Analyzer
┌────────────────────────────────────────────────────────────────┐
│  Dependencies (1-25 of 127)                                    │
├────────────────────────────────────────────────────────────────┤
│  ► my-app @1.0.0                                               │
│  ├── ▼ dependencies (45)                                       │
│  │   ├── [P] react @18.2.0              [45.00 KB (3.2%)]      │
│  │   ├── [P] react-dom @18.2.0          [120.00 KB (8.5%)]     │
│  │   ├── [P] lodash @4.17.21            [70.00 KB (5.0%)]      │
│  │   ├── [!] moment @2.29.4             [290.00 KB (20.6%)]    │
│  │   └── ...                                                   │
│  └── ▶ devDependencies (82)                                    │
└────────────────────────────────────────────────────────────────┘
│  / Search  s Sort  j/k Nav  q Quit  │  [P] Prod  [D] Dev       │
└────────────────────────────────────────────────────────────────┘
```

## Features

- **Fast Analysis** - Analyze 1000+ dependencies in under 2 seconds
- **Interactive TUI** - Vim-style navigation with collapsible tree view
- **Bundle Size Impact** - See which dependencies actually bloat your bundle
- **Circular Dependency Detection** - Find problematic dependency cycles
- **Version Conflict Detection** - Identify version mismatches across your tree
- **Savings Analysis** - Calculate potential bundle size savings
- **CI/CD Integration** - Exit codes and JSON output for automation
- **Cross-Platform** - Works on Linux, macOS, and Windows

## Installation

### From Cargo (Recommended)

```bash
cargo install codescope
```

### From Binary Releases

Download the latest release for your platform from the [Releases page](https://github.com/zach-fau/codescope/releases).

```bash
# Linux (x86_64)
curl -L https://github.com/zach-fau/codescope/releases/latest/download/codescope-linux-x86_64.tar.gz | tar xz
sudo mv codescope /usr/local/bin/

# macOS (Apple Silicon)
curl -L https://github.com/zach-fau/codescope/releases/latest/download/codescope-darwin-arm64.tar.gz | tar xz
sudo mv codescope /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/zach-fau/codescope/releases/latest/download/codescope-windows-x86_64.zip -OutFile codescope.zip
Expand-Archive codescope.zip -DestinationPath .
```

### Build from Source

```bash
git clone https://github.com/zach-fau/codescope.git
cd codescope
cargo build --release
./target/release/codescope --version
```

## Quick Start

```bash
# Navigate to your JavaScript/TypeScript project
cd your-project

# Launch interactive dependency tree
codescope analyze

# View dependencies without TUI (for scripting)
codescope analyze --no-tui

# Sort by bundle size impact
codescope analyze --sort-by-size
```

## Usage Examples

### Example 1: Basic Dependency Tree Analysis

Analyze your project's dependencies with an interactive, collapsible tree view:

```bash
cd my-react-app
codescope analyze
```

**Output:**
```
CodeScope - Dependency Analyzer
┌─────────────────────────────────────────────────────────────┐
│ Dependencies (47)                                           │
├─────────────────────────────────────────────────────────────┤
│ ► my-react-app @1.0.0                                       │
│ ├── ▼ dependencies (12)                                     │
│ │   ├── [P] react @18.2.0                                   │
│ │   ├── [P] react-dom @18.2.0                               │
│ │   ├── [P] react-router-dom @6.22.0                        │
│ │   ├── [P] axios @1.6.7                                    │
│ │   └── [P] zustand @4.5.0                                  │
│ └── ▶ devDependencies (35)                                  │
└─────────────────────────────────────────────────────────────┘
```

Use `j`/`k` or arrow keys to navigate, `Enter` to expand/collapse nodes.

### Example 2: Bundle Size Analysis

Analyze bundle size impact to identify heavy dependencies:

```bash
# Run your build first to generate stats
npm run build

# Analyze with bundle size sorting
codescope analyze --sort-by-size
```

**Output:**
```
┌─────────────────────────────────────────────────────────────────┐
│ Dependencies (sorted by size)                                   │
├─────────────────────────────────────────────────────────────────┤
│   [P] moment @2.29.4                    [290.00 KB (20.6%)]     │
│   [P] lodash @4.17.21                   [70.00 KB (5.0%)]       │
│   [P] react-dom @18.2.0                 [120.00 KB (8.5%)]      │
│   [P] axios @1.6.7                      [15.00 KB (1.1%)]       │
│   [P] react @18.2.0                     [45.00 KB (3.2%)]       │
└─────────────────────────────────────────────────────────────────┘
```

Press `s` to cycle through sort modes: Alphabetical, Size (descending), Size (ascending).

### Example 3: CI/CD Integration - Detecting Circular Dependencies

Use CodeScope in CI pipelines to catch circular dependencies early:

```bash
# Check for circular dependencies (exits with code 1 if found)
codescope analyze --check-cycles

# Example CI usage (GitHub Actions)
# .github/workflows/ci.yml
# - name: Check for circular dependencies
#   run: codescope analyze --check-cycles
```

**Success Output:**
```
✅ No circular dependencies detected.
```

**Failure Output:**
```
❌ Circular dependencies detected!

  Cycle 1: module-a -> module-b -> module-c -> module-a

Found 1 circular dependency cycle(s).
```

### Example 4: Savings Report for Bundle Optimization

Generate a report showing potential bundle size savings:

```bash
codescope analyze --savings-report
```

**Output:**
```
Bundle Size Savings Report
══════════════════════════════════════════════════════════════

Summary:
  Total Bundle Size:     1.41 MB
  Potential Savings:     412.50 KB (28.5%)

Breakdown:
  Unused Dependencies:   2 packages  (290.00 KB)
  Underutilized:         3 packages  (72.50 KB)
  Tree-Shakeable:        5 packages  (50.00 KB)

Top Savings Opportunities:
┌────────────────────────────────────────────────────────────┐
│ [U] moment                              -290.00 KB         │
│ [<] lodash (using 3/200 exports)        -45.00 KB          │
│ [T] date-fns (tree-shakeable)           -12.00 KB          │
└────────────────────────────────────────────────────────────┘

Recommendations:
  • Remove 'moment' - not imported anywhere in source
  • Replace 'lodash' with individual imports or lodash-es
```

### Example 5: CI Threshold Enforcement

Set a maximum threshold for potential savings to enforce bundle hygiene:

```bash
# Fail if potential savings exceed 100KB
codescope analyze --savings-report --savings-threshold 100

# Use in CI to prevent bundle bloat
```

**Failure Output:**
```
❌ Potential savings (412.50 KB) exceed threshold (100 KB)!
```

## CLI Reference

```
codescope [COMMAND] [OPTIONS]

COMMANDS:
    analyze     Analyze dependencies in the current project
    version     Show version information
    help        Print help information

ANALYZE OPTIONS:
    -p, --path <PATH>           Path to analyze [default: .]
    -w, --with-bundle-size      Include bundle size analysis
        --no-tui                Print dependency tree to stdout (no interactive UI)
        --check-cycles          Check for circular dependencies (CI mode, exits 1 if found)
        --check-conflicts       Check for version conflicts (CI mode, exits 1 if found)
        --sort-by-size          Sort dependencies by bundle size (largest first)
        --savings-report        Generate bundle size savings report
        --savings-threshold <KB> Set minimum savings threshold for CI checks
    -h, --help                  Print help information

EXAMPLES:
    codescope analyze                          # Interactive TUI in current directory
    codescope analyze --path ./my-project      # Analyze specific directory
    codescope analyze --no-tui                 # Output to stdout (for piping/scripting)
    codescope analyze --check-cycles           # CI check for circular dependencies
    codescope analyze --savings-report         # Show potential savings
```

## TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` / `Space` | Toggle expand/collapse |
| `g` / `Home` | Jump to first item |
| `G` / `End` | Jump to last item |
| `d` / `PageDown` | Page down |
| `u` / `PageUp` | Page up |
| `/` | Start search |
| `s` | Cycle sort mode (A-Z / Size↓ / Size↑) |
| `i` | Toggle savings panel (when available) |
| `Esc` | Clear search / Close panel / Quit |
| `q` | Quit |

### Search Mode

| Key | Action |
|-----|--------|
| Type | Filter dependencies (fuzzy match) |
| `↑` / `↓` | Navigate filtered results |
| `Enter` | Confirm search and exit search mode |
| `Esc` | Cancel search |
| `Backspace` | Delete last character |

## Dependency Type Legend

| Indicator | Type | Description |
|-----------|------|-------------|
| `[P]` | Production | Bundled with your application (green) |
| `[D]` | Development | Only needed during development (yellow) |
| `[Pe]` | Peer | Expected to be provided by consumer (cyan) |
| `[O]` | Optional | Enhances functionality if available (gray) |
| `[!]` | Cycle | Part of a circular dependency (red) |
| `[~]` | Conflict | Has version conflicts (orange) |

## Configuration

CodeScope works out of the box with zero configuration. Advanced configuration options can be set via a `.codescoperc` file in your project root (coming soon).

```json
{
  "exclude": ["node_modules", "dist"],
  "bundleStats": "./stats.json",
  "thresholds": {
    "maxBundleSize": "500KB",
    "maxDependencies": 100
  }
}
```

## How It Works

1. **Parsing**: CodeScope reads your `package.json` to understand project dependencies
2. **Graph Building**: Dependencies are organized into a directed graph using [petgraph](https://github.com/petgraph/petgraph)
3. **Analysis**: The graph is analyzed for cycles, conflicts, and size impact
4. **Visualization**: Results are rendered in an interactive TUI using [ratatui](https://github.com/ratatui-org/ratatui)

## Performance

| Metric | Value |
|--------|-------|
| Startup time | < 100ms |
| 100 dependencies | < 200ms |
| 1000 dependencies | < 2s |
| Memory usage | < 50MB |
| Binary size | < 10MB |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

```bash
# Clone and build
git clone https://github.com/zach-fau/codescope.git
cd codescope
cargo build

# Run tests
cargo test

# Run with test project
cargo run -- analyze --path test-project
```

## Architecture

For details on the internal architecture, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Roadmap

- [x] JavaScript/TypeScript dependency analysis
- [x] Interactive TUI with tree view
- [x] Bundle size visualization
- [x] Circular dependency detection
- [x] CI/CD integration
- [ ] Multi-language support (Python, Go, Rust)
- [ ] Watch mode for real-time updates
- [ ] Export to JSON/CSV/Markdown
- [ ] GitHub Actions marketplace action

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Built with Rust** | [Report an Issue](https://github.com/zach-fau/codescope/issues) | [Documentation](https://github.com/zach-fau/codescope#readme)
