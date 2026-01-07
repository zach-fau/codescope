# Next Steps - CodeScope Development

> **This document is the source of truth for what to do next.**
> Update this file at the end of each development session.

**Last Updated**: 2026-01-07T14:48:54Z
**Current Phase**: Week 1 - Foundation
**Status**: Setup Complete ✅ - Ready to Start Development

---

## 🎯 Immediate Next Action

**Start Week 1 Development**: Implement core dependency parsing

### First Task: Package.json Parser
**File to create**: `src/parser/package_json.rs`

**What to build**:
1. Create `PackageJson` struct with serde
2. Implement parsing logic for:
   - dependencies
   - devDependencies
   - peerDependencies
   - optionalDependencies
3. Add error handling for malformed JSON
4. Write unit tests

**Reference**: See TODO.md lines 9-16 for detailed checklist

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

### Day 1-2: Parser Implementation ← **WE ARE HERE**
- [ ] Implement package.json parser
- [ ] Add dependency structure types
- [ ] Write parser unit tests
- [ ] Test with real package.json files

### Day 3-4: Graph Implementation
- [ ] Build dependency graph with petgraph
- [ ] Add nodes for each package
- [ ] Create dependency edges
- [ ] Implement cycle detection

### Day 5-6: Basic TUI
- [ ] Setup ratatui terminal
- [ ] Create tree widget
- [ ] Add keyboard navigation
- [ ] Implement expand/collapse

### Day 7: Integration & Testing
- [ ] End-to-end test (parse → graph → display)
- [ ] Fix bugs
- [ ] Update documentation
- [ ] Prepare for Week 2

**Target by End of Week 1**: Working CLI that displays package.json dependencies as a tree

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

**Setup Phase Complete** ✅

What was done:
- [x] Created PRD
- [x] Created GitHub repository
- [x] Initialized local Rust project
- [x] Set up development environment (Rust, Cargo)
- [x] Created all documentation
- [x] Pushed to GitHub

What's ready:
- [x] Cargo.toml with all dependencies
- [x] Basic CLI structure with clap
- [x] Directory structure created
- [x] GitHub repo initialized
- [x] Documentation complete

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
