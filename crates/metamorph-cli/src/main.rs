use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use metamorph::{
    CompatibilityReport, ConvertRequest, Format, PublishRequest, Source, Target, acquire_source,
    cache_dir, compatibility, convert, inspect, plan, plan_publish, publish, validate,
};

#[derive(Debug, Parser)]
#[command(
    name = "metamorph",
    version,
    about = "Model format conversion utility for local-first AI runtimes"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Inspect {
        input: String,
    },
    Convert {
        #[arg(long)]
        input: String,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        from: Option<Format>,
        #[arg(long)]
        to: Format,
        #[arg(long)]
        allow_lossy: bool,
        #[arg(long)]
        plan_only: bool,
    },
    Validate {
        path: PathBuf,
        #[arg(long)]
        format: Option<Format>,
    },
    Upload {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        repo: String,
        #[arg(long)]
        execute: bool,
    },
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
}

#[derive(Debug, Subcommand)]
enum CacheCommand {
    Dir,
    Source {
        input: String,
        #[arg(long)]
        from: Option<Format>,
        #[arg(long)]
        materialize: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Inspect { input } => inspect_command(&input),
        Command::Convert {
            input,
            output,
            from,
            to,
            allow_lossy,
            plan_only,
        } => convert_command(&input, output, from, to, allow_lossy, plan_only),
        Command::Validate { path, format } => validate_command(&path, format),
        Command::Upload {
            input,
            repo,
            execute,
        } => upload_command(&input, &repo, execute),
        Command::Cache { command } => cache_command(command),
    }
}

fn inspect_command(input: &str) -> Result<()> {
    let source =
        Source::from_str(input).with_context(|| format!("failed to parse source `{input}`"))?;
    let report = inspect(&source)?;

    println!("Source: {}", report.source);
    match report.detected_format {
        Some(format) => println!("Detected format: {format}"),
        None => println!("Detected format: unknown"),
    }

    if !report.notes.is_empty() {
        println!("Notes:");
        for note in report.notes {
            println!("- {note}");
        }
    }

    Ok(())
}

fn convert_command(
    input: &str,
    output: PathBuf,
    from: Option<Format>,
    to: Format,
    allow_lossy: bool,
    plan_only: bool,
) -> Result<()> {
    let source =
        Source::from_str(input).with_context(|| format!("failed to parse source `{input}`"))?;
    let request = ConvertRequest {
        source,
        target: Target::LocalDir(output),
        from,
        to,
        allow_lossy,
    };
    let compatibility_report = compatibility(&request)?;
    print_compatibility_report(&compatibility_report);
    let conversion_plan = plan(&request)?;

    println!(
        "Planned conversion: {} -> {}",
        conversion_plan.source_format, conversion_plan.target_format
    );
    println!("Target: {}", conversion_plan.target);
    println!("Execution: {}", conversion_plan.execution);
    if let Some(backend) = &conversion_plan.backend {
        println!("Backend: {backend}");
    }
    println!("Lossy: {}", conversion_plan.lossy);
    println!("Steps:");
    for step in &conversion_plan.steps {
        println!("- {step}");
    }
    if !conversion_plan.notes.is_empty() {
        println!("Notes:");
        for note in &conversion_plan.notes {
            println!("- {note}");
        }
    }

    if plan_only {
        return Ok(());
    }

    convert(&request)?;
    println!("Converted bundle: {}", request.target);

    Ok(())
}

fn print_compatibility_report(report: &CompatibilityReport) {
    println!("Compatibility status: {}", report.status);
    if let Some(source_format) = report.source_format {
        println!("Resolved source format: {source_format}");
    }
    println!("Requested target format: {}", report.target_format);
    println!("Lossy: {}", report.lossy);
    if let Some(backend) = &report.backend {
        println!("Compatible backend: {backend}");
    }
    if !report.blockers.is_empty() {
        println!("Blockers:");
        for blocker in &report.blockers {
            println!("- {blocker}");
        }
    }
    if !report.notes.is_empty() {
        println!("Compatibility notes:");
        for note in &report.notes {
            println!("- {note}");
        }
    }
}

fn validate_command(path: &Path, expected: Option<Format>) -> Result<()> {
    let report = validate(path, expected)?;
    println!("Validated {} as {}", path.display(), report.format);
    println!("Reusable: {}", report.reusable);
    if !report.checked_paths.is_empty() {
        println!("Checked paths:");
        for checked_path in report.checked_paths {
            println!("- {}", checked_path.display());
        }
    }
    if !report.notes.is_empty() {
        println!("Notes:");
        for note in report.notes {
            println!("- {note}");
        }
    }

    Ok(())
}

fn upload_command(input: &Path, repo: &str, execute: bool) -> Result<()> {
    let plan = plan_publish(input, repo)?;

    println!(
        "Planned publish: {} -> {}",
        plan.input.display(),
        plan.destination
    );
    println!("Validated format: {}", plan.validated_format);
    println!("Artifacts:");
    for artifact in &plan.artifacts {
        println!("- {}", artifact.display());
    }
    println!("Steps:");
    for step in &plan.steps {
        println!("- {step}");
    }

    let report = publish(&PublishRequest {
        input: input.to_path_buf(),
        target: Target::HuggingFaceRepo(repo.to_owned()),
        execute,
    })?;
    println!("Executed: {}", report.executed);
    if !report.notes.is_empty() {
        println!("Notes:");
        for note in report.notes {
            println!("- {note}");
        }
    }

    Ok(())
}

fn cache_command(command: CacheCommand) -> Result<()> {
    match command {
        CacheCommand::Dir => {
            println!("{}", cache_dir().display());
            Ok(())
        }
        CacheCommand::Source {
            input,
            from,
            materialize,
        } => {
            let source = Source::from_str(&input)
                .with_context(|| format!("failed to parse source `{input}`"))?;
            let report = acquire_source(&source, from, materialize)?;
            println!("Source: {}", report.source);
            match report.detected_format {
                Some(format) => println!("Detected format: {format}"),
                None => println!("Detected format: unknown"),
            }
            println!("Cache key: {}", report.cache_identity.key);
            println!("Cache path: {}", report.cache_identity.path.display());
            println!("Status: {}", report.outcome);
            println!("Resolved path: {}", report.resolved_path.display());
            if !report.notes.is_empty() {
                println!("Notes:");
                for note in report.notes {
                    println!("- {note}");
                }
            }
            Ok(())
        }
    }
}
