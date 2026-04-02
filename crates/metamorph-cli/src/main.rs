use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use metamorph::{ConvertRequest, Format, Source, Target, convert, inspect, plan};

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
    },
    Cache {
        #[command(subcommand)]
        command: CacheCommand,
    },
}

#[derive(Debug, Subcommand)]
enum CacheCommand {
    Dir,
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
        Command::Upload { input, repo } => upload_command(&input, &repo),
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
    let conversion_plan = plan(&request)?;

    println!(
        "Planned conversion: {} -> {}",
        conversion_plan.source_format, conversion_plan.target_format
    );
    println!("Target: {}", conversion_plan.target);
    println!("Lossy: {}", conversion_plan.lossy);
    println!("Steps:");
    for step in &conversion_plan.steps {
        println!("- {step}");
    }

    if plan_only {
        return Ok(());
    }

    convert(&request).map_err(Into::into)
}

fn validate_command(path: &Path, expected: Option<Format>) -> Result<()> {
    let source = Source::LocalPath(path.to_path_buf());
    let report = inspect(&source)?;

    let detected = report.detected_format;
    if let Some(expected) = expected {
        match detected {
            Some(detected) if detected == expected => {
                println!("Validated {} as {expected}", path.display());
            }
            Some(detected) => {
                bail!(
                    "expected {} to be {}, but detected {}",
                    path.display(),
                    expected,
                    detected
                );
            }
            None => {
                bail!("could not determine the format for {}", path.display());
            }
        }
    } else {
        match detected {
            Some(format) => println!("Detected {} as {format}", path.display()),
            None => bail!("could not determine the format for {}", path.display()),
        }
    }

    Ok(())
}

fn upload_command(input: &PathBuf, repo: &str) -> Result<()> {
    let _ = (input, repo);
    bail!("upload support is not implemented yet");
}

fn cache_command(command: CacheCommand) -> Result<()> {
    match command {
        CacheCommand::Dir => {
            let cache_dir = std::env::var("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(".cache"))
                .join("metamorph");
            println!("{}", cache_dir.display());
            Ok(())
        }
    }
}
