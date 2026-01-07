# CodeScope Project Guide

> **This file explains how to work on CodeScope across multiple Claude sessions**

## 🎯 Quick Start for Any Session

### What to Say to New Agent

**Copy-paste this to start a new session:**

```
I'm working on CodeScope, a Rust terminal dependency analyzer.
Read /home/gyatso/Development/codescope/.claude/CLAUDE.md first,
then check NEXT_STEPS.md and current GitHub issues. Let's continue
development from where we left off.
```

### Then the Agent Will:

```bash
# 1. Read the workflow guide
cat /home/gyatso/Development/codescope/.claude/CLAUDE.md  # This file

# 2. Check what's next
cat /home/gyatso/Development/codescope/NEXT_STEPS.md

# 3. Review GitHub issues
gh issue list --repo zach-fau/codescope

# 4. Start working
cd /home/gyatso/Development/codescope
```

---

## 📍 Project Organization

### Core Philosophy
**Use GitHub for task management, local docs for reference.**

- ✅ **GitHub Issues**: Track features, bugs, tasks
- ✅ **GitHub Projects**: Visual progress board
- ✅ **NEXT_STEPS.md**: What to work on right now
- ✅ **TODO.md**: Reference checklist (less important than GitHub)
- ❌ **CCPM /pm commands**: Don't use (outdated, doesn't fit workflow)

### Why GitHub Issues?
- Survives context resets
- Visible to collaborators
- Integrates with code (PRs, commits)
- Provides discussion threads
- Can track progress with labels/milestones

---

## 📂 File Locations

### Project Files
```
/home/gyatso/Development/codescope/
├── .claude/
│   └── CLAUDE.md              ← You are here (workflow guide)
├── src/                       ← Source code
├── tests/                     ← Tests
├── NEXT_STEPS.md             ← What to work on next (living doc)
├── TODO.md                   ← Reference checklist
├── ARCHITECTURE.md           ← System design
├── CONTRIBUTING.md           ← Dev workflow
└── Cargo.toml                ← Dependencies
```

### PRD & Research
```
/home/gyatso/Development/career-planning/
└── .claude/
    ├── pm/prds/codescope.md        ← PRD (4-week plan, resume value)
    └── research/
        ├── final-recommendations-2026-01-07.md
        └── combined-analysis-2026-01-07.md
```

### GitHub
```
Repository: https://github.com/zach-fau/codescope
Issues:     https://github.com/zach-fau/codescope/issues
Projects:   https://github.com/zach-fau/codescope/projects
```

---

## 🔄 Workflow for Each Session

### 1. Starting a Session

```bash
# Check current status
cd /home/gyatso/Development/codescope
git status
git pull origin main

# Read what's next
cat NEXT_STEPS.md

# Check GitHub issues
gh issue list --state open --assignee @me

# Pick an issue or check NEXT_STEPS.md for guidance
```

### 2. Working on a Task

**Option A: From GitHub Issue**
```bash
# View issue
gh issue view <number>

# Create branch (optional, not required for MVP)
git checkout -b issue-<number>-description

# Work on code, commit frequently
git add -A
git commit -m "feat: implement package.json parser (#<issue-number>)"
git push origin main

# Close issue when done
gh issue close <number> -c "Completed in commit abc123"
```

**Option B: From NEXT_STEPS.md**
```bash
# Follow the "Immediate Next Action" in NEXT_STEPS.md
# Create GitHub issue if it doesn't exist
gh issue create --title "Implement package.json parser" \
  --body "Week 1 task: Parse package.json and extract dependencies"

# Work and commit
```

### 3. Ending a Session

**Critical**: Update continuity documents!

```bash
# 1. Update NEXT_STEPS.md
# - Set new "Immediate Next Action"
# - Update "Last Updated" timestamp
# - Mark current phase/status

# 2. Update GitHub (if using issues)
# - Close completed issues
# - Update issue comments with progress
# - Create new issues for discovered work

# 3. Commit and push
git add -A
git commit -m "docs: update NEXT_STEPS for next session"
git push origin main
```

---

## 📋 Task Management Strategy

### Use GitHub Issues for:
- ✅ Features to implement (e.g., "Implement package.json parser")
- ✅ Bugs to fix
- ✅ Tasks spanning multiple sessions
- ✅ Anything requiring discussion or tracking

### Use NEXT_STEPS.md for:
- ✅ "What should I work on RIGHT NOW?"
- ✅ Current session context
- ✅ Quick continuity between agents
- ✅ Immediate next action (links to GitHub issue)

### Use TODO.md for:
- ✅ Reference: Week-by-week breakdown
- ✅ High-level checklist
- ⚠️ Don't rely on it as source of truth (GitHub is better)

### Don't Use CCPM /pm commands:
- ❌ Outdated (6 months old)
- ❌ Doesn't align with modern Claude Code
- ❌ GitHub provides better features

---

## 🎯 Week-by-Week Workflow

### Week 1 Example (Current)

**Monday** (Jan 7):
```bash
# Setup phase (done!)
# Created GitHub repo, local project, docs
```

**Tuesday** (Jan 8):
```bash
# Create GitHub issue for parser
gh issue create --title "Week 1: Implement package.json parser" \
  --body "Parse dependencies, devDependencies, peerDependencies from package.json" \
  --label "week-1,parser"

# Work on parser
# Update NEXT_STEPS.md when done
```

**Wednesday-Thursday**:
```bash
# Continue Week 1 tasks
# Check off items in TODO.md
# Update GitHub issues
```

**Friday** (Jan 13):
```bash
# Week 1 review
gh issue list --milestone "Week 1"

# Prepare for Week 2
# Update NEXT_STEPS.md with Week 2 starting point
```

---

## 🔗 Integration with GitHub

### Creating Issues from TODO.md

The TODO.md has task breakdowns. Convert these to GitHub issues:

```bash
# Example: Week 1 parser tasks
gh issue create --title "Parse package.json dependencies" \
  --body "Implement parser for dependencies, devDependencies, peerDependencies" \
  --label "week-1,parser,core"

gh issue create --title "Add dependency graph structure" \
  --body "Use petgraph to build dependency graph from parsed data" \
  --label "week-1,graph,core"

gh issue create --title "Implement basic TUI tree widget" \
  --body "Create collapsible tree widget with ratatui" \
  --label "week-1,ui,tui"
```

### Using Milestones

Create milestones for each week:
```bash
gh api repos/zach-fau/codescope/milestones \
  -f title="Week 1: Foundation" \
  -f due_on="2026-01-13T23:59:59Z"

# Assign issues to milestone
gh issue edit <number> --milestone "Week 1: Foundation"
```

---

## 🧭 Decision Making Guide

### "Should I create a GitHub issue or just update NEXT_STEPS.md?"

**Create GitHub issue if**:
- Task will span multiple sessions
- Task is a distinct feature/bug
- You want to track progress publicly
- Multiple people might work on it

**Just update NEXT_STEPS.md if**:
- Quick fix or small change
- Immediate next action for current session
- Linking to existing GitHub issue

### "Should I use GitHub Projects?"

**Yes, if**:
- You want visual board (Kanban)
- Managing multiple features in parallel
- Want to see overall progress

**No, if**:
- Simple linear workflow (Week 1 → Week 2 → etc.)
- GitHub issues + NEXT_STEPS.md are enough

---

## 📚 Reference Documents

### Read These for Context
| Document | Purpose | When to Read |
|----------|---------|--------------|
| **PRD** | Project goals, timeline, success criteria | Start of project, when stuck |
| **ARCHITECTURE.md** | System design, technical decisions | Before implementing features |
| **CONTRIBUTING.md** | Code style, commit format, PR process | Before committing code |
| **NEXT_STEPS.md** | What to do RIGHT NOW | Start of every session |

### Update These During Work
| Document | When to Update |
|----------|----------------|
| **NEXT_STEPS.md** | End of every session (set next action) |
| **CHANGELOG.md** | When shipping features |
| **TODO.md** | (Optional) Check off items as reference |

---

## 🛠️ Common Commands Reference

### Git & GitHub
```bash
# Status and sync
git status
git pull origin main
git push origin main

# Issues
gh issue list
gh issue create --title "..." --body "..."
gh issue view <number>
gh issue close <number>

# Repo info
gh repo view zach-fau/codescope
```

### Rust Development
```bash
# Use full path or source env first
~/.cargo/bin/cargo build
~/.cargo/bin/cargo test
~/.cargo/bin/cargo clippy
~/.cargo/bin/cargo fmt

# Run the CLI
~/.cargo/bin/cargo run -- analyze
```

---

## 📊 Progress Tracking

### How to Know Where You Are

1. **Check NEXT_STEPS.md** - "Current Phase" section
2. **Check GitHub milestones** - `gh api repos/zach-fau/codescope/milestones`
3. **Check TODO.md** - See what's checked off
4. **Check CHANGELOG.md** - See what's been shipped

### How to Report Progress

```bash
# Update NEXT_STEPS.md
# Set "Last Updated" to current date
# Update "Current Phase"
# Update "Status"

# Update GitHub
gh issue comment <number> -b "Completed X, next: Y"

# Commit changes
git add -A
git commit -m "docs: update progress for Week 1"
git push origin main
```

---

## 🎓 Best Practices

### Do's ✅
- ✅ Read NEXT_STEPS.md at start of every session
- ✅ Create GitHub issues for features
- ✅ Commit frequently with clear messages
- ✅ Update NEXT_STEPS.md at end of session
- ✅ Test before committing (`cargo test`)
- ✅ Reference PRD when making decisions

### Don'ts ❌
- ❌ Don't use CCPM /pm commands (outdated)
- ❌ Don't leave NEXT_STEPS.md outdated
- ❌ Don't work without reading context first
- ❌ Don't forget to push to GitHub
- ❌ Don't create issues without checking existing ones

---

## 🚀 Quick Workflow Summary

```
START SESSION
  ├─ Read NEXT_STEPS.md
  ├─ Check GitHub issues (gh issue list)
  ├─ Pull latest (git pull)
  └─ Pick task

WORK
  ├─ Create/update GitHub issue
  ├─ Write code
  ├─ Test (cargo test)
  ├─ Commit (git commit)
  └─ Push (git push)

END SESSION
  ├─ Update NEXT_STEPS.md (new "Immediate Next Action")
  ├─ Update GitHub issues (close/comment)
  ├─ Commit updates
  └─ Push to GitHub
```

---

## 📞 Help & Resources

### Where to Find Answers
- **Technical questions**: ARCHITECTURE.md
- **What to build**: PRD (`/home/gyatso/Development/career-planning/.claude/pm/prds/codescope.md`)
- **How to work**: This file (.claude/CLAUDE.md)
- **What's next**: NEXT_STEPS.md

### Research Context
- Market analysis: `/home/gyatso/Development/career-planning/.claude/research/`
- Why CodeScope: See PRD "Differentiation" section
- Resume value: 6.5/10 (Rust + TUI + build tooling)

---

**Last Updated**: 2026-01-07
**Status**: Week 1 - Ready to start development
**Next**: Implement package.json parser (create GitHub issue first)
