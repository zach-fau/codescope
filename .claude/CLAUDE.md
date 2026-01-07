# CodeScope

Terminal UI dependency analyzer with bundle size impact visualization. Built in Rust with ratatui.

## Current State

**Lint**: ✅ | **Build**: ✅ | **Tests**: 50 passing

### What's Next

**Check GitHub Issues** for current tasks:
```bash
gh issue list --repo zach-fau/codescope
```

If no open issues, reference the PRD for what to work on next:
- PRD: `/home/gyatso/Development/career-planning/.claude/pm/prds/codescope.md`

---

## Project Structure

```
codescope/
├── src/
│   ├── main.rs           # CLI entry point, integration
│   ├── parser/           # package.json parsing
│   │   ├── mod.rs
│   │   ├── types.rs      # PackageJson, Dependency, DependencyType
│   │   └── package_json.rs
│   ├── graph/            # Dependency graph (petgraph)
│   │   ├── mod.rs
│   │   └── dependency_graph.rs
│   └── ui/               # TUI (ratatui)
│       ├── mod.rs
│       ├── app.rs        # App state, event loop
│       └── tree.rs       # TreeNode, FlattenedNode
├── test-project/         # Sample package.json for testing
├── Cargo.toml
└── docs/
    ├── ARCHITECTURE.md
    └── CONTRIBUTING.md
```

---

## Commands

```bash
# Build & Test
~/.cargo/bin/cargo build
~/.cargo/bin/cargo test
~/.cargo/bin/cargo clippy

# Run
~/.cargo/bin/cargo run -- analyze --path <directory>
~/.cargo/bin/cargo run -- analyze --path test-project --no-tui
```

### TUI Keyboard Shortcuts
- `j`/`↓` - Move down
- `k`/`↑` - Move up
- `Enter`/`Space` - Toggle expand/collapse
- `q`/`Esc` - Quit

---

## Technical Decisions

- **Rust** - Performance, cross-platform binaries
- **ratatui** - Modern TUI framework (successor to tui-rs)
- **petgraph** - Graph data structure for dependencies
- **serde** - JSON parsing for package.json
- **clap** - CLI argument parsing

---

## Reference

| Doc | Purpose |
|-----|---------|
| PRD | 4-week timeline, features, success metrics |
| ARCHITECTURE.md | System design, data flow |
| CONTRIBUTING.md | Code style, commit format |

**GitHub**: https://github.com/zach-fau/codescope
