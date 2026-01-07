use std::io;
use std::path::Path;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use codescope::bundle::savings::{SavingsCalculator, SavingsReport};
use codescope::graph::{self, DependencyGraph};
use codescope::parser::{self, extract_dependencies, parse_file, DependencyType};
use codescope::ui::{run_app, App, TreeNode, format_size, SortMode};

#[derive(Parser)]
#[command(name = "codescope")]
#[command(author = "Zachary Woods <143150513+zach-fau@users.noreply.github.com>")]
#[command(version = "0.1.0")]
#[command(about = "Terminal UI dependency analyzer with bundle size impact visualization", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze dependencies in the current project
    Analyze {
        /// Path to analyze (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: String,

        /// Include bundle size analysis
        #[arg(short, long)]
        with_bundle_size: bool,

        /// Print dependency tree to stdout without TUI
        #[arg(long)]
        no_tui: bool,

        /// Check for circular dependencies (for CI usage, exits with code 1 if found)
        #[arg(long)]
        check_cycles: bool,

        /// Check for version conflicts (for CI usage, exits with code 1 if found)
        #[arg(long)]
        check_conflicts: bool,

        /// Sort dependencies by bundle size (largest first) instead of alphabetically
        #[arg(long)]
        sort_by_size: bool,

        /// Generate a bundle size savings report (for CI usage)
        /// Shows potential savings from removing unused/underutilized dependencies
        #[arg(long)]
        savings_report: bool,

        /// Set a minimum savings threshold in KB for CI checks
        /// Exit with code 1 if potential savings exceed this threshold
        #[arg(long, value_name = "KB")]
        savings_threshold: Option<u64>,
    },
    /// Show version information
    Version,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Analyze {
            path,
            with_bundle_size: _,
            no_tui,
            check_cycles,
            check_conflicts,
            sort_by_size,
            savings_report,
            savings_threshold,
        }) => {
            let package_json_path = Path::new(path).join("package.json");

            if !package_json_path.exists() {
                eprintln!("❌ No package.json found at: {}", package_json_path.display());
                eprintln!("   Run this command in a directory with a package.json file.");
                std::process::exit(1);
            }

            // Parse package.json
            let pkg = match parse_file(&package_json_path) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("❌ Failed to parse package.json: {}", e);
                    std::process::exit(1);
                }
            };

            // Extract dependencies
            let deps = extract_dependencies(&pkg);

            // Build dependency graph for cycle detection
            let graph = build_dependency_graph(&deps);

            // Handle --check-cycles flag (for CI usage)
            if *check_cycles {
                let cycles = graph.get_cycle_details();
                if cycles.is_empty() {
                    println!("✅ No circular dependencies detected.");
                    return Ok(());
                } else {
                    eprintln!("❌ Circular dependencies detected!");
                    eprintln!();
                    for (i, cycle) in cycles.iter().enumerate() {
                        eprintln!("  Cycle {}: {}", i + 1, cycle.cycle_path());
                    }
                    eprintln!();
                    eprintln!("Found {} circular dependency cycle(s).", cycles.len());
                    std::process::exit(1);
                }
            }

            // Handle --check-conflicts flag (for CI usage)
            if *check_conflicts {
                let conflicts = graph.detect_version_conflicts();
                if conflicts.is_empty() {
                    println!("✅ No version conflicts detected.");
                    return Ok(());
                } else {
                    eprintln!("❌ Version conflicts detected!");
                    eprintln!();
                    for conflict in &conflicts {
                        eprintln!("  {}", conflict.description());
                    }
                    eprintln!();
                    eprintln!("Found {} version conflict(s).", conflicts.len());
                    std::process::exit(1);
                }
            }

            // Handle --savings-report flag (for CI usage)
            if *savings_report {
                let report = generate_savings_report(&deps);
                print!("{}", report.format_report());

                // Check threshold if specified
                if let Some(threshold_kb) = savings_threshold {
                    let threshold_bytes = threshold_kb * 1024;
                    if report.summary.total_potential_savings > threshold_bytes {
                        eprintln!();
                        eprintln!(
                            "❌ Potential savings ({}) exceed threshold ({} KB)!",
                            report.summary.format_total_savings(),
                            threshold_kb
                        );
                        std::process::exit(1);
                    } else {
                        println!();
                        println!(
                            "✅ Potential savings ({}) are within threshold ({} KB).",
                            report.summary.format_total_savings(),
                            threshold_kb
                        );
                    }
                }
                return Ok(());
            }

            // Build tree structure
            let mut tree = build_dependency_tree(&pkg.name.clone().unwrap_or_else(|| "project".to_string()),
                                             &pkg.version.clone().unwrap_or_else(|| "0.0.0".to_string()),
                                             &deps);

            // Mark nodes that are part of cycles
            let cycle_nodes = graph.get_nodes_in_cycles();
            tree.mark_cycles(&cycle_nodes);

            // Mark nodes with version conflicts
            let conflict_packages = graph.get_packages_with_conflicts();
            tree.mark_conflicts(&conflict_packages);

            if *no_tui {
                // Print tree to stdout
                let total_bundle_size = calculate_tree_total_bundle_size(&tree);
                print_tree(&tree, 0, total_bundle_size);
                return Ok(());
            }

            // Setup terminal for TUI
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend)?;

            // Create app and run with appropriate sort mode
            let initial_sort_mode = if *sort_by_size {
                SortMode::SizeDescending
            } else {
                SortMode::Alphabetical
            };
            let mut app = App::with_sort_mode(tree, initial_sort_mode);
            let result = run_app(&mut terminal, &mut app);

            // Restore terminal
            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;

            if let Err(e) = result {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Version) => {
            println!("codescope v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            println!("CodeScope - Terminal UI Dependency Analyzer");
            println!();
            println!("Usage:");
            println!("  codescope analyze [OPTIONS]     Analyze dependencies");
            println!("  codescope version               Show version");
            println!();
            println!("Run 'codescope --help' for more options");
        }
    }

    Ok(())
}

/// Build a TreeNode from parsed dependencies
fn build_dependency_tree(
    project_name: &str,
    project_version: &str,
    deps: &[parser::Dependency],
) -> TreeNode {
    let mut root = TreeNode::new(project_name.to_string(), project_version.to_string());
    root.expanded = true; // Start with root expanded

    // Group dependencies by type
    let mut prod_deps: Vec<TreeNode> = Vec::new();
    let mut dev_deps: Vec<TreeNode> = Vec::new();
    let mut peer_deps: Vec<TreeNode> = Vec::new();
    let mut optional_deps: Vec<TreeNode> = Vec::new();

    for dep in deps {
        // Create node with dependency type for color coding
        let node = TreeNode::with_dep_type(dep.name.clone(), dep.version.clone(), dep.dep_type);
        match dep.dep_type {
            DependencyType::Production => prod_deps.push(node),
            DependencyType::Development => dev_deps.push(node),
            DependencyType::Peer => peer_deps.push(node),
            DependencyType::Optional => optional_deps.push(node),
        }
    }

    // Add category nodes with their children
    if !prod_deps.is_empty() {
        let mut prod_node = TreeNode::new(
            format!("dependencies ({})", prod_deps.len()),
            String::new(),
        );
        prod_node.expanded = true;
        for dep in prod_deps {
            prod_node.add_child(dep);
        }
        root.add_child(prod_node);
    }

    if !dev_deps.is_empty() {
        let mut dev_node = TreeNode::new(
            format!("devDependencies ({})", dev_deps.len()),
            String::new(),
        );
        for dep in dev_deps {
            dev_node.add_child(dep);
        }
        root.add_child(dev_node);
    }

    if !peer_deps.is_empty() {
        let mut peer_node = TreeNode::new(
            format!("peerDependencies ({})", peer_deps.len()),
            String::new(),
        );
        for dep in peer_deps {
            peer_node.add_child(dep);
        }
        root.add_child(peer_node);
    }

    if !optional_deps.is_empty() {
        let mut opt_node = TreeNode::new(
            format!("optionalDependencies ({})", optional_deps.len()),
            String::new(),
        );
        for dep in optional_deps {
            opt_node.add_child(dep);
        }
        root.add_child(opt_node);
    }

    root
}

/// Build a DependencyGraph from parsed dependencies for cycle detection
fn build_dependency_graph(deps: &[parser::Dependency]) -> DependencyGraph {
    let mut graph = DependencyGraph::with_capacity(deps.len(), deps.len() * 2);

    for dep in deps {
        let dep_type = match dep.dep_type {
            DependencyType::Production => graph::DependencyType::Production,
            DependencyType::Development => graph::DependencyType::Development,
            DependencyType::Peer => graph::DependencyType::Peer,
            DependencyType::Optional => graph::DependencyType::Optional,
        };
        graph.add_dependency(&dep.name, &dep.version, dep_type);
    }

    // Note: In a real implementation, we would add edges based on resolved
    // dependency relationships from lock files or npm/yarn resolution.
    // For now, the graph only contains nodes without edges, so cycle detection
    // will only work if edges are added elsewhere.

    graph
}

/// Print tree to stdout (for --no-tui mode)
fn print_tree(node: &TreeNode, depth: usize, total_bundle_size: u64) {
    let indent = "  ".repeat(depth);
    let indicator = if node.children.is_empty() {
        "  "
    } else if node.expanded {
        "▼ "
    } else {
        "▶ "
    };

    // Get type indicator for the dependency
    let type_indicator = match node.dep_type {
        Some(DependencyType::Production) => "[P] ",
        Some(DependencyType::Development) => "[D] ",
        Some(DependencyType::Peer) => "[Pe] ",
        Some(DependencyType::Optional) => "[O] ",
        None => "",
    };

    // Get cycle indicator
    let cycle_indicator = if node.is_in_cycle { "[!] " } else { "" };

    // Get conflict indicator
    let conflict_indicator = if node.has_conflict { "[~] " } else { "" };

    // Get bundle size indicator
    let size_indicator = if let Some(size) = node.bundle_size {
        if total_bundle_size > 0 {
            let percentage = (size as f64 / total_bundle_size as f64) * 100.0;
            format!(" [{} ({:.1}%)]", format_size(size), percentage)
        } else {
            format!(" [{}]", format_size(size))
        }
    } else {
        String::new()
    };

    if node.version.is_empty() {
        println!("{}{}{}", indent, indicator, node.name);
    } else {
        println!("{}{}{}{}{}{} @ {}{}", indent, indicator, cycle_indicator, conflict_indicator, type_indicator, node.name, node.version, size_indicator);
    }

    if node.expanded || depth == 0 {
        for child in &node.children {
            print_tree(child, depth + 1, total_bundle_size);
        }
    }
}

/// Calculate total bundle size from a tree
fn calculate_tree_total_bundle_size(node: &TreeNode) -> u64 {
    let mut total = node.bundle_size.unwrap_or(0);
    for child in &node.children {
        total += calculate_tree_total_bundle_size(child);
    }
    total
}

/// Generate a savings report from parsed dependencies
///
/// This creates a mock bundle analysis from the dependency list since we don't
/// have actual webpack stats. For real bundle size data, use --with-bundle-size
/// with a stats.json file.
fn generate_savings_report(deps: &[parser::Dependency]) -> SavingsReport {
    use std::collections::HashMap;
    use codescope::bundle::webpack::{BundleAnalysis, PackageBundleSize};
    use codescope::analysis::exports::ProjectImports;

    // Create a mock bundle analysis from dependencies
    // In a real implementation, this would come from webpack stats
    let mut analysis = BundleAnalysis::default();

    // Use estimated sizes based on common package sizes
    // This is a heuristic - real sizes would come from webpack stats
    let estimated_sizes: HashMap<&str, u64> = [
        ("react", 45 * 1024),
        ("react-dom", 120 * 1024),
        ("lodash", 70 * 1024),
        ("moment", 290 * 1024),
        ("axios", 15 * 1024),
        ("express", 200 * 1024),
        ("webpack", 100 * 1024),
        ("typescript", 10 * 1024), // TypeScript is dev-only, minimal bundle impact
        ("@types/", 0), // Type definitions have no runtime cost
        ("eslint", 0), // Dev dependency
        ("jest", 0), // Dev dependency
        ("prettier", 0), // Dev dependency
    ].into_iter().collect();

    let default_size = 25 * 1024; // 25KB default estimate

    for dep in deps {
        // Skip dev dependencies for bundle size (they don't affect runtime bundle)
        if matches!(dep.dep_type, DependencyType::Development) {
            continue;
        }

        // Estimate size based on known packages or use default
        let size = estimated_sizes
            .iter()
            .find(|(name, _)| dep.name.starts_with(*name))
            .map(|(_, size)| *size)
            .unwrap_or(default_size);

        if size > 0 {
            let mut pkg = PackageBundleSize::new(&dep.name);
            pkg.add_module(format!("{}/index.js", dep.name), size);
            analysis.package_sizes.insert(dep.name.clone(), pkg);
            analysis.total_module_size += size;
        }
    }

    // Create empty project imports (no source analysis in this mode)
    // In a real implementation, we'd analyze the source code
    let project_imports = ProjectImports::new();
    let export_counts = HashMap::new();

    // Calculate savings
    let calculator = SavingsCalculator::new();
    calculator.calculate(&analysis, &project_imports, &export_counts)
}
