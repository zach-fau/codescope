# CodeScope TODO

## Week 1: Foundation (Jan 7-13, 2026)

### Core Dependencies
- [ ] Implement package.json parser
  - [ ] Parse dependencies field
  - [ ] Parse devDependencies field
  - [ ] Parse peerDependencies field
  - [ ] Handle version ranges
  - [ ] Error handling for malformed JSON

- [ ] Build dependency graph
  - [ ] Create graph data structure (petgraph)
  - [ ] Add nodes for each package
  - [ ] Add edges for dependencies
  - [ ] Detect circular dependencies
  - [ ] Calculate dependency depth

### TUI Development
- [ ] Setup ratatui framework
  - [ ] Initialize terminal
  - [ ] Handle terminal cleanup
  - [ ] Basic event loop

- [ ] Create tree widget
  - [ ] Render tree structure
  - [ ] Collapsible nodes
  - [ ] Color-coded dependency types
  - [ ] Scroll support

- [ ] Keyboard navigation
  - [ ] Arrow keys for navigation
  - [ ] Enter to expand/collapse
  - [ ] vim-like bindings (j/k)
  - [ ] 'q' to quit

### Testing
- [ ] Parser unit tests
- [ ] Graph unit tests
- [ ] Integration test for basic workflow

**Target**: Working dependency tree parser and basic TUI

---

## Week 2: Enhanced Visualization (Jan 14-20, 2026)

### Search & Filter
- [ ] Implement search functionality
  - [ ] Search by package name
  - [ ] Fuzzy search
  - [ ] Highlight matches
  - [ ] Navigate between matches

- [ ] Add filtering
  - [ ] Filter by dependency type
  - [ ] Filter by version
  - [ ] Filter by depth level

### Dependency Analysis
- [ ] Circular dependency detection
  - [ ] Find cycles using Tarjan's algorithm
  - [ ] Highlight cycles in UI
  - [ ] Export cycle report

- [ ] Version conflict detection
  - [ ] Identify same package with different versions
  - [ ] Show conflicts in UI
  - [ ] Suggest resolutions

### UI Enhancements
- [ ] Color scheme improvements
  - [ ] Distinguish dev/peer/optional dependencies
  - [ ] Highlight circular dependencies
  - [ ] Theme configuration

- [ ] Status bar
  - [ ] Total packages count
  - [ ] Circular dependencies warning
  - [ ] Current filter/search state

**Target**: Interactive TUI with search, filtering, and cycle detection

---

## Week 3: Bundle Size Analysis (Jan 21-27, 2026)

### Bundle Integration
- [ ] Parse webpack-bundle-analyzer JSON
  - [ ] Load stats.json file
  - [ ] Extract module information
  - [ ] Map modules to packages

- [ ] Parse vite-bundle-visualizer output
  - [ ] Load stats file
  - [ ] Extract bundle information

### Size Calculation
- [ ] Map dependencies to bundle sizes
  - [ ] Attribute size to each package
  - [ ] Handle shared modules
  - [ ] Calculate transitive size

- [ ] Display size in UI
  - [ ] Add size column to tree
  - [ ] Sort by size
  - [ ] Show percentage of total
  - [ ] Format sizes (KB, MB)

### Export Analysis
- [ ] Detect unused exports (tree-sitter)
  - [ ] Parse source files
  - [ ] Find exported symbols
  - [ ] Check usage across codebase
  - [ ] Calculate potential savings

**Target**: Bundle size impact shown alongside dependency tree

---

## Week 4: Polish & Release (Jan 28 - Feb 3, 2026)

### Export Functionality
- [ ] Export to JSON
  - [ ] Full dependency graph
  - [ ] Include size data
  - [ ] Format for machine reading

- [ ] Export to CSV
  - [ ] Package name, version, size
  - [ ] Flat structure for spreadsheets

- [ ] Export to Markdown
  - [ ] Human-readable format
  - [ ] Include recommendations
  - [ ] Format for documentation

### Configuration
- [ ] Configuration file support (.codescoperc)
  - [ ] Default ignored packages
  - [ ] Custom thresholds
  - [ ] Color scheme customization
  - [ ] Default analysis options

### CI/CD Integration
- [ ] Exit codes for thresholds
  - [ ] Fail on circular dependencies
  - [ ] Fail on bundle size > threshold
  - [ ] Fail on unused dependencies count

- [ ] GitHub Actions example
  - [ ] Sample workflow file
  - [ ] Integration documentation

### Documentation
- [ ] Comprehensive README
  - [ ] Installation instructions
  - [ ] Usage examples
  - [ ] Screenshots/GIFs
  - [ ] FAQ section

- [ ] ARCHITECTURE.md ✅
  - [ ] Update with implementation details

- [ ] API documentation
  - [ ] Generate with rustdoc
  - [ ] Publish to docs.rs

### Performance
- [ ] Benchmarking
  - [ ] Create benchmark suite
  - [ ] Test with large projects
  - [ ] Optimize hot paths

- [ ] Memory optimization
  - [ ] Profile memory usage
  - [ ] Reduce allocations
  - [ ] Streaming where possible

### Cross-Platform
- [ ] Test on Linux ✅
- [ ] Test on macOS
- [ ] Test on Windows
- [ ] Fix platform-specific issues

### Release
- [ ] Version 1.0.0 release
  - [ ] Create GitHub release
  - [ ] Build binaries (Linux, macOS, Windows)
  - [ ] Publish to crates.io
  - [ ] Update documentation

**Target**: Production-ready v1.0.0 release

---

## Post-MVP (Phase 2+)

### Multi-Language Support
- [ ] Python (requirements.txt, pyproject.toml)
- [ ] Go (go.mod)
- [ ] Rust (Cargo.toml with full analysis)

### Advanced Features
- [ ] Real-time watch mode
- [ ] Dependency update recommendations
- [ ] Security vulnerability scanning
- [ ] Visual graph export (SVG/PNG)

### Distribution
- [ ] Homebrew formula
- [ ] Debian/Ubuntu package
- [ ] AUR package
- [ ] Chocolatey package (Windows)

### Ecosystem
- [ ] VS Code extension
- [ ] GitHub Action
- [ ] Pre-commit hook

---

**Last Updated**: January 7, 2026
**Current Phase**: Week 1 - Foundation
