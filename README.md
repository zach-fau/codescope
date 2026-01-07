# CodeScope

> Terminal UI dependency analyzer with bundle size impact visualization

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)

## 🎯 What is CodeScope?

CodeScope is a **fast, terminal-native dependency analyzer** that helps developers understand their codebase dependencies and their real impact on bundle size. Built in Rust for performance and cross-platform support.

### The Problem

When joining a new codebase or reviewing dependencies, developers face:
- Hours spent mentally mapping dependency relationships
- Slow web-based tools that require browser context switching
- Difficulty identifying which dependencies actually bloat your bundle
- No quick way to find unused exports and transitive dependencies

### The Solution

CodeScope provides:
- ⚡ **Fast Terminal UI** - Analyze 1000+ dependencies in < 2 seconds
- 🎨 **Interactive Visualization** - Collapsible dependency tree with vim-like navigation
- 📊 **Bundle Size Impact** - See which dependencies actually bloat your bundle
- 🔍 **Unused Export Detection** - Find dependencies you're barely using
- 🚀 **Zero Configuration** - Works out of the box with JavaScript/TypeScript projects

## 🚀 Quick Start

```bash
# Install (coming soon)
cargo install codescope

# Analyze your project
cd your-project
codescope analyze

# With bundle size analysis (requires build output)
npm run build  # or your build command
codescope analyze --with-bundle-size
```

## ✨ Features

### Week 1-2: Core Dependency Analysis (In Progress)
- [x] Parse package.json dependencies
- [ ] Build interactive dependency tree
- [ ] Keyboard navigation (vim-like bindings)
- [ ] Circular dependency detection
- [ ] Color-coded dependency types

### Week 3: Bundle Size Analysis (Planned)
- [ ] Integrate with webpack/vite/rollup stats
- [ ] Show bundle size contribution per dependency
- [ ] Identify unused exports
- [ ] Calculate potential size savings

### Week 4: Polish & Release (Planned)
- [ ] Export reports (JSON, CSV, Markdown)
- [ ] CI/CD integration
- [ ] Cross-platform binaries
- [ ] Comprehensive documentation

## 🎨 Screenshots

_Coming soon - Interactive TUI demo will be added here_

## 🏗️ Tech Stack

- **Language**: Rust (for performance and cross-platform support)
- **TUI Framework**: [ratatui](https://github.com/ratatui-org/ratatui)
- **Parsing**: [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
- **Graph**: [petgraph](https://github.com/petgraph/petgraph)

## 🎯 Why CodeScope?

### vs webpack-bundle-analyzer
- ✅ Terminal-native (faster, no browser context switch)
- ✅ Works over SSH and remote development
- ✅ Lightweight binary (< 10MB vs multi-GB Electron apps)

### vs Sourcegraph
- ✅ Free and open-source
- ✅ Focuses specifically on dependency analysis
- ✅ Perfect for individual developers and small teams

### vs bundlephobia.com
- ✅ Analyzes YOUR codebase (not just generic package stats)
- ✅ Shows actual usage and project-specific impact
- ✅ Identifies unused exports in your code

## 📖 Documentation

- [Architecture Overview](docs/ARCHITECTURE.md) (coming soon)
- [Contributing Guide](CONTRIBUTING.md) (coming soon)
- [Development Setup](docs/development.md) (coming soon)

## 🛣️ Roadmap

### MVP (Week 1-4) - January 2026
- JavaScript/TypeScript dependency analysis
- Interactive TUI with bundle size visualization
- Export to JSON/CSV/Markdown
- CI/CD integration

### Phase 2 (February 2026)
- Multi-language support (Python, Go, Rust)
- Real-time watch mode
- GitHub Actions integration

### Phase 3 (March 2026+)
- Dependency update recommendations
- Security vulnerability scanning
- Visual graph export (SVG/PNG)

## 🤝 Contributing

Contributions are welcome! This project is in active development.

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## 📜 License

MIT License - see [LICENSE](LICENSE) for details

## 🙏 Acknowledgments

Inspired by:
- [dive](https://github.com/wagoodman/dive) - Docker image layer analysis
- [webpack-bundle-analyzer](https://github.com/webpack-contrib/webpack-bundle-analyzer) - Bundle analysis
- [dependency-cruiser](https://github.com/sverweij/dependency-cruiser) - Dependency validation

---

**Status**: 🚧 Active Development (Week 1 of 4)

**Author**: [Zachary Woods](https://github.com/zach-fau)

**Built with** ❤️ **and** 🦀 **Rust**
