# Next Steps - CodeScope Development

> **This document is the source of truth for what to do next.**
> Update this file at the end of each development session.

**Last Updated**: 2026-01-07T15:52:55Z
**Current Phase**: Week 1 - Foundation
**Status**: Day 1-2 Complete ✅ - Parser, Graph, TUI Implemented

---

## 🎯 Immediate Next Action

**Continue Week 1**: Enhance the TUI and add search/filtering

### Next Task: Search & Filter Functionality
**Files to modify**: `src/ui/app.rs`, `src/ui/tree.rs`

**What to build**:
1. Add search input field to TUI
2. Implement fuzzy filtering of dependencies
3. Highlight matching text in tree
4. Add keyboard shortcut `/` to start search
5. Add `Esc` to clear search

### Alternative: Dependency Type Indicators
**Files to modify**: `src/ui/app.rs`

**What to build**:
1. Add color-coding for dependency types
2. Production = green, Development = yellow, Peer = cyan, Optional = gray
3. Show type indicator (P/D/Pe/O) next to each dependency

---

## 📍 Project Locations

### Key Files & Directories

**Project Root**: `/home/gyatso/Development/codescope/`
```
codescope/
├── src/main.rs                    # CLI entry point (basic structure done)
├── src/parser/                    # ← START HERE (empty, needs implementation)
├── src/ui/                        # TUI components (empty)
├── src/graph/                     # Dependency graph logic (empty)
├── src/cli/                       # CLI logic (empty)
├── Cargo.toml                     # Dependencies configured ✅
├── TODO.md                        # Week-by-week task breakdown ✅
├── ARCHITECTURE.md                # System design reference ✅
└── NEXT_STEPS.md                  # This file (always update!)
```

**PRD Location**: `/home/gyatso/Development/career-planning/.claude/pm/prds/codescope.md`

**GitHub Repo**: https://github.com/zach-fau/codescope

---

## 📋 Progress Tracking System

### Primary Documents (Update These)

1. **NEXT_STEPS.md** (this file)
   - **Purpose**: What to do next, current status
   - **Update**: At the end of every session
   - **Location**: `/home/gyatso/Development/codescope/NEXT_STEPS.md`

2. **TODO.md**
   - **Purpose**: Week-by-week task breakdown
   - **Update**: Check off completed tasks, add new discoveries
   - **Location**: `/home/gyatso/Development/codescope/TODO.md`

3. **CHANGELOG.md**
   - **Purpose**: Record all changes (follows Keep a Changelog format)
   - **Update**: When significant features are added
   - **Location**: `/home/gyatso/Development/codescope/CHANGELOG.md`

### Reference Documents (Read, Don't Update)

- **PRD**: Complete project spec and timeline
- **ARCHITECTURE.md**: System design and technical decisions
- **CONTRIBUTING.md**: Development workflow and conventions

---

## 🔄 Workflow for Next Agent/Session

### Step 1: Context Check
```bash
# Read this file first
cat /home/gyatso/Development/codescope/NEXT_STEPS.md

# Check current git status
cd /home/gyatso/Development/codescope
git status
git log --oneline -5

# Review current TODO items
cat TODO.md | grep "Week 1" -A 20
```

### Step 2: Start Development
Follow the "Immediate Next Action" section above.

### Step 3: Development Cycle
```bash
# Make changes
# Test changes
~/.cargo/bin/cargo test

# Build
~/.cargo/bin/cargo build

# Commit
git add -A
git commit -m "feat(parser): implement package.json parser"
git push origin main
```

### Step 4: Update Tracking (IMPORTANT!)
Before ending session:
1. ✅ Update TODO.md - check off completed items
2. ✅ Update CHANGELOG.md - add to [Unreleased] section
3. ✅ **Update NEXT_STEPS.md** - Set new "Immediate Next Action"

---

## 📅 Week 1 Timeline (Jan 7-13, 2026)

### Day 1-2: Parser Implementation ✅ COMPLETE
- [x] Implement package.json parser
- [x] Add dependency structure types
- [x] Write parser unit tests (21 tests)
- [x] Test with real package.json files

### Day 3-4: Graph Implementation ✅ COMPLETE
- [x] Build dependency graph with petgraph
- [x] Add nodes for each package
- [x] Create dependency edges
- [x] Implement cycle detection (17 tests)

### Day 5-6: Basic TUI ✅ COMPLETE
- [x] Setup ratatui terminal
- [x] Create tree widget (12 tests)
- [x] Add keyboard navigation (j/k/↑/↓)
- [x] Implement expand/collapse (Enter/Space)

### Day 7: Integration & Testing ← **WE ARE HERE** (ahead of schedule!)
- [x] End-to-end test (parse → graph → display)
- [ ] Add search/filter functionality
- [ ] Add color-coded dependency types
- [ ] Prepare for Week 2

**Target by End of Week 1**: Working CLI that displays package.json dependencies as a tree ✅ ACHIEVED

---

## 🛠️ Development Commands

### Build & Test
```bash
# Source Rust environment
source ~/.cargo/env
# OR use full path
~/.cargo/bin/cargo build

# Run tests
~/.cargo/bin/cargo test

# Run the CLI
~/.cargo/bin/cargo run -- analyze

# Check for errors
~/.cargo/bin/cargo clippy

# Format code
~/.cargo/bin/cargo fmt
```

### Git Workflow
```bash
cd /home/gyatso/Development/codescope

# Check status
git status

# Commit changes
git add -A
git commit -m "type(scope): description"
git push origin main
```

---

## 🎯 Week 1 Success Criteria

By end of Week 1, we should have:
- ✅ package.json parser working
- ✅ Dependency graph built with petgraph
- ✅ Basic TUI displaying tree structure
- ✅ Keyboard navigation working
- ✅ Unit tests passing
- ✅ Can analyze a real JavaScript project

---

## 📊 Handoff Checklist for Current Session

**Week 1 Day 1-2 Complete** ✅

What was done this session:
- [x] Implemented package.json parser (src/parser/)
- [x] Implemented dependency graph with petgraph (src/graph/)
- [x] Created TUI tree widget with ratatui (src/ui/)
- [x] Integrated all modules in main.rs
- [x] Added --no-tui flag for stdout output
- [x] Wrote 50 unit tests (all passing)
- [x] Pushed to GitHub, closed Issue #1

What's working:
- [x] `codescope analyze --path <dir>` - launches interactive TUI
- [x] `codescope analyze --no-tui` - prints tree to stdout
- [x] Keyboard navigation: j/k/↑/↓, Enter/Space to expand, q to quit
- [x] Groups dependencies by type (prod, dev, peer, optional)

---

## 💡 Quick Tips for Next Agent

1. **Always read NEXT_STEPS.md first** - It's your starting point
2. **Update NEXT_STEPS.md before ending** - Set the next action
3. **Use full cargo path** (`~/.cargo/bin/cargo`) or source env first
4. **Check TODO.md** for detailed task breakdown
5. **Reference ARCHITECTURE.md** for design decisions
6. **Follow conventional commits** (feat, fix, docs, test, chore)
7. **Test before committing** - Run `cargo test` and `cargo clippy`
8. **Update CHANGELOG.md** when adding features

---

## 🚨 Important Notes

### Rust Environment
- Rust installed at: `~/.cargo/`
- Either source env: `source ~/.cargo/env`
- Or use full path: `~/.cargo/bin/cargo`

### Project Structure
- Main code: `src/`
- Tests: `tests/`
- Documentation: `*.md` files in root
- No code written yet (just setup)

### Git Status
- Remote: https://github.com/zach-fau/codescope.git
- Branch: main
- Last commit: Documentation added
- Status: Clean, ready for development

---

## 🔗 Quick Links

- **Project Repo**: https://github.com/zach-fau/codescope
- **Research**: `/home/gyatso/Development/career-planning/.claude/research/`
- **PRD**: `/home/gyatso/Development/career-planning/.claude/pm/prds/codescope.md`
- **TODO**: `/home/gyatso/Development/codescope/TODO.md`
- **Architecture**: `/home/gyatso/Development/codescope/ARCHITECTURE.md`

---

**Status**: ✅ Ready to start Week 1 development
**Next Session**: Implement package.json parser (src/parser/package_json.rs)
