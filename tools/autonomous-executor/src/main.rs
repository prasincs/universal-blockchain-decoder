//! Autonomous Task Executor
//!
//! Self-improving automation that continuously executes highest ROI tasks
//! from the roadmap until full test coverage is achieved.

mod cli;
mod executor;
mod git;
mod parser;
mod roi;
mod task;

use anyhow::Result;
use clap::Parser;
use log::{error, info};

use crate::cli::{Cli, Command};
use crate::executor::TaskExecutor;
use crate::parser::RoadmapParser;
use crate::roi::RoiCalculator;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Command::ListTasks { roadmap_path } => {
            list_tasks(&roadmap_path)?;
        }
        Command::Execute {
            roadmap_path,
            task_id,
            top_n,
            dry_run,
        } => {
            execute_tasks(&roadmap_path, task_id, top_n, dry_run).await?;
        }
    }

    Ok(())
}

/// List all tasks sorted by ROI
fn list_tasks(roadmap_path: &str) -> Result<()> {
    info!("Parsing roadmap: {}", roadmap_path);

    let parser = RoadmapParser::new(roadmap_path)?;
    let mut tasks = parser.parse()?;

    // Calculate ROI for all tasks
    let calculator = RoiCalculator::new();
    for task in &mut tasks {
        task.roi_score = calculator.calculate(task);
    }

    // Sort by ROI (descending)
    tasks.sort_by(|a, b| {
        b.roi_score
            .partial_cmp(&a.roi_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("\nTasks (sorted by ROI):\n");

    for (i, task) in tasks.iter().take(20).enumerate() {
        println!("{}. {}: {}", i + 1, task.id, task.title);
        println!(
            "   Status: {} | Priority: {} | ROI: {:.2}",
            task.status, task.priority, task.roi_score
        );
        println!(
            "   Time: {} | Completed: {}/{}",
            task.time_estimate.as_ref().unwrap_or(&"N/A".to_string()),
            task.completed_items.len(),
            task.completed_items.len() + task.remaining_items.len()
        );
        println!();
    }

    Ok(())
}

/// Execute tasks based on selection criteria
async fn execute_tasks(
    roadmap_path: &str,
    task_id: Option<String>,
    top_n: usize,
    dry_run: bool,
) -> Result<()> {
    println!("{}", "=".repeat(80));
    println!("Universal Blockchain Decoder - Autonomous Task Executor");
    println!("{}", "=".repeat(80));
    println!();

    // Parse roadmap
    info!("[1/5] Parsing roadmap...");
    let parser = RoadmapParser::new(roadmap_path)?;
    let mut tasks = parser.parse()?;
    info!("Found {} tasks", tasks.len());

    // Calculate ROI
    let calculator = RoiCalculator::new();
    for task in &mut tasks {
        task.roi_score = calculator.calculate(task);
    }

    // Sort by ROI
    tasks.sort_by(|a, b| {
        b.roi_score
            .partial_cmp(&a.roi_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Select tasks
    info!("\n[2/5] Selecting tasks...");
    let selected_tasks: Vec<_> = if let Some(id) = task_id {
        tasks.into_iter().filter(|t| t.id == id).collect()
    } else {
        tasks
            .into_iter()
            .filter(|t| t.status != "✅ COMPLETE")
            .take(top_n)
            .collect()
    };

    if selected_tasks.is_empty() {
        info!("No tasks to execute (all completed or none match criteria)");
        return Ok(());
    }

    info!("Selected {} task(s) for execution:", selected_tasks.len());
    for task in &selected_tasks {
        info!(
            "  - {} (ROI: {:.2}, Priority: {})",
            task.title, task.roi_score, task.priority
        );
    }

    // Execute tasks
    info!("\n[3/5] Executing tasks...");
    let executor = TaskExecutor::new()?;

    let mut results = Vec::new();
    for task in selected_tasks {
        let result = executor.execute(&task, dry_run).await;
        results.push((task, result));
    }

    // Report results
    info!("\n[5/5] Execution Summary:");
    println!("{}", "=".repeat(80));

    let successful = results.iter().filter(|(_, r)| r.is_ok()).count();
    let failed = results.len() - successful;

    println!(
        "\nTotal: {} | Successful: {} | Failed: {}\n",
        results.len(),
        successful,
        failed
    );

    for (task, result) in results {
        match result {
            Ok(message) => {
                info!("✅ SUCCESS: {}", task.title);
                info!("  {}\n", message);
            }
            Err(e) => {
                error!("❌ FAILED: {}", task.title);
                error!("  {}\n", e);
            }
        }
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
