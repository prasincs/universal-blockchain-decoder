//! CLI argument parsing

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "autonomous-executor")]
#[command(about = "Autonomous task executor with ROI-based prioritization", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all tasks sorted by ROI
    ListTasks {
        /// Path to ROADMAP.md
        #[arg(short, long, default_value = "ROADMAP.md")]
        roadmap_path: String,
    },

    /// Execute tasks
    Execute {
        /// Path to ROADMAP.md
        #[arg(short, long, default_value = "ROADMAP.md")]
        roadmap_path: String,

        /// Execute specific task ID (e.g., 'phase-3.2')
        #[arg(short, long)]
        task_id: Option<String>,

        /// Execute top N highest ROI tasks
        #[arg(short = 'n', long, default_value = "1")]
        top_n: usize,

        /// Dry run (preview without executing)
        #[arg(short, long)]
        dry_run: bool,
    },
}
