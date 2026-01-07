# Contributing to CodeScope

Thank you for your interest in contributing to CodeScope! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites
- Rust 1.92.0 or later
- Git
- A terminal emulator that supports color output

### Setup Instructions

1. **Clone the repository**
   ```bash
   git clone https://github.com/zach-fau/codescope.git
   cd codescope
   ```

2. **Install Rust** (if not already installed)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

3. **Build the project**
   ```bash
   cargo build
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

5. **Run the CLI**
   ```bash
   cargo run -- analyze
   ```

## Project Structure

```
codescope/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── parser/           # Dependency manifest parsers
│   ├── ui/               # TUI components (ratatui)
│   ├── graph/            # Dependency graph logic
│   └── cli/              # Command-line interface logic
├── tests/                # Integration tests
├── benches/              # Performance benchmarks
├── docs/                 # Additional documentation
└── examples/             # Example usage
```

## Development Workflow

### 1. Pick an Issue
- Check [GitHub Issues](https://github.com/zach-fau/codescope/issues)
- Comment on the issue to claim it
- For major changes, open an issue first to discuss

### 2. Create a Branch
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 3. Make Changes
- Follow Rust naming conventions
- Add tests for new functionality
- Update documentation as needed
- Run `cargo fmt` before committing
- Run `cargo clippy` to catch common mistakes

### 4. Test Your Changes
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Check code quality
cargo clippy
```

### 5. Commit Your Changes
Follow conventional commit format:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```bash
git commit -m "feat(parser): add Cargo.toml parser"
git commit -m "fix(ui): correct tree rendering bug"
git commit -m "docs: update installation instructions"
```

### 6. Push and Create Pull Request
```bash
git push origin your-branch-name
```

Then open a pull request on GitHub with:
- Clear description of changes
- Reference to related issue (if applicable)
- Screenshots/GIFs for UI changes

## Code Style

### Rust Style Guidelines
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common issues
- Prefer explicit over implicit
- Add documentation comments for public APIs

### Documentation
- Use `///` for public API documentation
- Include examples in documentation where helpful
- Keep README.md updated with major features

### Testing
- Write unit tests for new functions
- Write integration tests for user-facing features
- Aim for meaningful test coverage (not just high numbers)
- Test edge cases and error conditions

## Pull Request Review Process

1. **Automated Checks**
   - Tests must pass
   - Code must compile
   - Clippy lints must pass
   - Format check must pass

2. **Code Review**
   - At least one maintainer approval required
   - Address review feedback
   - Keep discussions professional and constructive

3. **Merge**
   - Squash commits if requested
   - Maintainer will merge when ready

## Development Priorities (Week 1-4)

### Week 1-2: Foundation
- Package.json parser
- Dependency graph structure
- Basic TUI tree widget
- Keyboard navigation

### Week 3: Bundle Size
- Webpack stats parser
- Bundle size integration
- Unused export detection

### Week 4: Polish
- Export functionality
- Configuration file support
- Cross-platform testing
- Documentation

## Getting Help

- **Questions**: Open a [GitHub Discussion](https://github.com/zach-fau/codescope/discussions)
- **Bugs**: Open a [GitHub Issue](https://github.com/zach-fau/codescope/issues)
- **Security**: Email security@codescope.dev (coming soon)

## Code of Conduct

Be respectful and inclusive. We're all here to build something great together.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

**Thank you for contributing to CodeScope!** 🦀
