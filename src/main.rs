use clap::{Parser, Subcommand};

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
    },
    /// Show version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Analyze {
            path,
            with_bundle_size,
        }) => {
            println!("🔍 Analyzing dependencies in: {}", path);
            if *with_bundle_size {
                println!("📊 Bundle size analysis enabled");
            }
            println!("\n⚠️  CodeScope is under development!");
            println!("✅ Week 1: Core dependency parsing (in progress)");
        }
        Some(Commands::Version) => {
            println!("codescope v{}", env!("CARGO_PKG_VERSION"));
        }
        None => {
            println!("CodeScope - Terminal UI Dependency Analyzer");
            println!("Run 'codescope analyze' to analyze dependencies");
            println!("Run 'codescope --help' for more information");
        }
    }
}
