use anyhow::Result;
use clap::{ArgAction, Parser};
use llm_client::{Effort, GenerationParams, LlmClient, Provider, ThinkingMode};
use log::{debug, info};
use std::path::PathBuf;

mod analyzer;
mod bedrock_api;
mod claude_api;
mod diagram_generator;
mod doc_updater;
mod git_utils;

#[derive(Parser, Debug)]
#[command(name = "auto-update-docs")]
#[command(about = "Automatically update documentation and architecture diagrams using Claude AI", long_about = None)]
struct Args {
    /// Repository root directory
    #[arg(long, default_value = ".")]
    repo_root: PathBuf,

    /// Anthropic API key (or set ANTHROPIC_API_KEY env var)
    /// Not required if using --use-bedrock
    #[arg(long, env)]
    api_key: Option<String>,

    /// Use AWS Bedrock instead of direct Anthropic API (often cheaper!)
    #[arg(long)]
    use_bedrock: bool,

    /// AWS region for Bedrock (e.g., us-east-1, us-west-2)
    #[arg(long, default_value = "us-east-1")]
    aws_region: String,

    /// Output directory for updated documentation
    #[arg(long, default_value = "docs")]
    docs_dir: PathBuf,

    /// Generate architecture diagrams
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    generate_diagrams: bool,

    /// Update existing documentation
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    update_docs: bool,

    /// Only analyze changes since this git ref (branch, tag, or commit)
    #[arg(long)]
    since: Option<String>,

    /// Create a git commit with the changes
    #[arg(long)]
    commit: bool,

    /// Push changes to remote
    #[arg(long)]
    push: bool,

    /// Create a pull request with changes
    #[arg(long)]
    create_pr: bool,

    /// Model to use
    /// For Anthropic API: claude-sonnet-4-6, claude-opus-4-8, claude-haiku-4-5
    /// For Bedrock: global.anthropic.claude-sonnet-4-6
    #[arg(long)]
    model: Option<String>,

    /// Maximum tokens for the model response
    #[arg(long, default_value = "8000")]
    max_tokens: u32,

    /// Sampling temperature (ignored when --thinking is set, or on models
    /// that reject sampling parameters)
    #[arg(long, default_value = "0.3")]
    temperature: f32,

    /// Enable adaptive thinking (model decides when and how much to reason)
    #[arg(long)]
    thinking: bool,

    /// Reasoning effort: low, medium, high, or max
    #[arg(long)]
    effort: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dry run (don't write files)
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logger
    if args.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    info!("🚀 Starting documentation auto-update tool");
    info!("Repository: {:?}", args.repo_root);

    // Create rate limiter for API calls
    let rate_limiter = claude_api::create_rate_limiter();
    info!("📊 Rate limiter configured: 10 RPM, 100K tokens/day, $10/month limit");

    // Normalized generation parameters (mapped per provider by llm-client)
    let effort = args
        .effort
        .as_deref()
        .map(|s| {
            Effort::parse(s)
                .ok_or_else(|| anyhow::anyhow!("invalid --effort '{}': use low|medium|high|max", s))
        })
        .transpose()?;
    let gen_params = GenerationParams {
        max_tokens: args.max_tokens,
        temperature: Some(args.temperature),
        thinking: if args.thinking {
            ThinkingMode::Adaptive
        } else {
            ThinkingMode::Off
        },
        effort,
    };

    // Determine which API to use and validate configuration
    let (api_mode, model) = if args.use_bedrock {
        info!("Using AWS Bedrock (region: {})", args.aws_region);

        // Check if Bedrock is available
        if let Err(e) = bedrock_api::check_bedrock_available() {
            anyhow::bail!("AWS Bedrock not available: {}", e);
        }

        let model = args
            .model
            .unwrap_or_else(|| "global.anthropic.claude-sonnet-4-6".to_string());

        info!("Bedrock model: {}", model);
        ("bedrock", model)
    } else {
        info!("Using Anthropic API");

        // Validate API key
        if args.api_key.is_none() {
            anyhow::bail!(
                "ANTHROPIC_API_KEY must be set or --api-key provided when not using Bedrock"
            );
        }

        let model = args
            .model
            .unwrap_or_else(|| "claude-sonnet-4-6".to_string());

        info!("Anthropic model: {}", model);
        ("anthropic", model)
    };

    // Rate-limited, cost-tracked client (Anthropic mode; Bedrock has its own path)
    let llm = if api_mode == "anthropic" {
        Some(
            LlmClient::new(Provider::anthropic(args.api_key.as_deref().unwrap()))
                .with_limiter(rate_limiter.clone()),
        )
    } else {
        None
    };

    // Step 1: Analyze the codebase
    info!("📊 Analyzing codebase structure...");
    let analysis = analyzer::analyze_codebase(&args.repo_root, args.since.as_deref())?;

    debug!(
        "Analysis complete: {} modules, {} recent changes",
        analysis.module_count,
        analysis.changes.len()
    );

    // Step 2: Determine what documentation needs updating
    info!("🔍 Determining documentation updates needed...");
    let updates_needed = doc_updater::determine_updates(&args.repo_root, &analysis)?;

    if updates_needed.is_empty() {
        info!("✅ All documentation is up to date!");
        return Ok(());
    }

    info!(
        "📝 Found {} documentation items to update",
        updates_needed.len()
    );

    // Step 3: Generate architecture diagrams using Claude
    if args.generate_diagrams {
        info!("🎨 Generating architecture diagrams...");
        let diagrams = if api_mode == "bedrock" {
            diagram_generator::generate_diagrams_bedrock(
                &args.aws_region,
                &model,
                args.max_tokens,
                args.temperature,
                &analysis,
            )?
        } else {
            diagram_generator::generate_diagrams_anthropic(
                llm.as_ref().unwrap(),
                &model,
                &gen_params,
                &analysis,
            )?
        };

        if !args.dry_run {
            for (name, diagram) in diagrams {
                diagram_generator::write_diagram(&args.docs_dir, &name, &diagram)?;
                info!("  ✓ Generated {}", name);
            }
        } else {
            info!(
                "  (Dry run: {} diagrams would be generated)",
                diagrams.len()
            );
        }
    }

    // Step 4: Update documentation using Claude
    if args.update_docs {
        info!("📚 Updating documentation...");
        for update in &updates_needed {
            info!("  Updating {}...", update.doc_path.display());

            let updated_content = if api_mode == "bedrock" {
                bedrock_api::generate_doc_update(
                    &args.aws_region,
                    &model,
                    args.max_tokens,
                    args.temperature,
                    update,
                    &analysis,
                )?
            } else {
                claude_api::generate_doc_update(
                    llm.as_ref().unwrap(),
                    &model,
                    &gen_params,
                    update,
                    &analysis,
                )?
            };

            if !args.dry_run {
                doc_updater::write_doc(&update.doc_path, &updated_content)?;
                info!("  ✓ Updated {}", update.doc_path.display());
            }
        }
    }

    // Step 5: Git operations
    if !args.dry_run {
        if args.commit {
            info!("📦 Creating git commit...");
            let commit_msg = format!(
                "docs: Auto-update documentation and diagrams\n\n\
                 Updated {} documentation files\n\
                 Generated {} architecture diagrams\n\n\
                 Generated by auto-update-docs tool",
                updates_needed.len(),
                if args.generate_diagrams {
                    "multiple"
                } else {
                    "no"
                }
            );
            git_utils::commit_changes(&args.repo_root, &commit_msg)?;
            info!("  ✓ Committed changes");

            if args.push {
                info!("⬆️  Pushing to remote...");
                git_utils::push_changes(&args.repo_root)?;
                info!("  ✓ Pushed changes");
            }

            if args.create_pr {
                info!("🔀 Creating pull request...");
                git_utils::create_pull_request(&args.repo_root, &commit_msg)?;
                info!("  ✓ Created pull request");
            }
        }
    } else {
        info!("(Dry run: no changes written)");
    }

    info!("✨ Documentation update complete!");

    Ok(())
}
