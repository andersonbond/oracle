use clap::{Parser, Subcommand, ValueEnum};
use oracle_guide::{
    Result, VERSION,
    agent::{self, Client},
    config::{self, Audience, Config, Kind, Stage},
    evidence::{self, Method, Outcome},
    rules::{self, Finding, Report, Status},
    terminal_safe,
};
use serde::Serialize;
use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Parser)]
#[command(
    name = "oracle",
    version,
    about = "Engineering guidance for vibe coders and their coding agents",
    long_about = "Oracle connects project decisions to release evidence. Local, offline, and read-only except explicit init, agent installation, and evidence recording. No project scripts are executed."
)]
struct Cli {
    #[arg(
        short = 'C',
        long,
        global = true,
        default_value = ".",
        help = "Existing project directory"
    )]
    root: PathBuf,
    #[arg(long, global = true, value_enum, default_value = "text")]
    format: Format,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a small project specification; preserve existing files.
    Init {
        #[arg(long)]
        name: Option<String>,
        #[arg(long, value_enum, default_value = "web-app")]
        kind: Kind,
        #[arg(long, value_enum, default_value = "prototype")]
        stage: Stage,
        #[arg(long, value_enum, default_value = "unspecified")]
        audience: Audience,
        #[arg(long)]
        dry_run: bool,
    },
    /// Inspect project decisions, repository hygiene, and evidence.
    Assess,
    /// Show a small number of useful next actions.
    Next {
        #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..=13))]
        limit: u8,
    },
    /// Explain a rule, trade-offs, action, and acceptance criteria.
    Explain { id: String },
    /// List the bundled versioned guidance cards.
    Rules,
    /// Assess; optionally fail on unresolved stage-appropriate requirements.
    Check {
        #[arg(long, help = "Return exit 1 for unmet applicable required checks")]
        strict: bool,
    },
    /// Install or remove the project-local agent skill without overwriting files.
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    /// Record supplied evidence with artifact hashes and project freshness.
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommand,
    },
}

#[derive(Subcommand)]
enum AgentCommand {
    Install {
        #[arg(value_enum)]
        client: Client,
        #[arg(long)]
        dry_run: bool,
    },
    Remove {
        #[arg(value_enum)]
        client: Client,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum EvidenceCommand {
    Record {
        id: String,
        #[arg(long, value_enum)]
        result: Outcome,
        #[arg(long, value_enum, default_value = "manual")]
        method: Method,
        #[arg(long)]
        summary: String,
        #[arg(
            long,
            required = true,
            help = "Project-relative sanitized report/screenshot (repeat up to 8 times)"
        )]
        artifact: Vec<PathBuf>,
    },
}

fn json(value: &impl Serialize) -> Result<()> {
    let mut out = io::stdout().lock();
    serde_json::to_writer_pretty(&mut out, value)?;
    writeln!(out)?;
    Ok(())
}

fn label(status: Status) -> &'static str {
    match status {
        Status::Pass => "pass",
        Status::Fail => "fail",
        Status::Unknown => "unknown",
        Status::Stale => "stale",
        Status::NotApplicable => "not applicable",
        Status::Deferred => "deferred",
    }
}

fn show_finding(f: &Finding, detail: bool) {
    println!(
        "{:<14} {:<10} {}{}",
        label(f.status),
        f.id,
        f.title,
        if f.required {
            " [required in strict mode]"
        } else {
            ""
        }
    );
    if detail {
        println!("  {}", terminal_safe(&f.detail));
    }
}

fn show_report(report: &Report) {
    println!(
        "Oracle {} · {} · {:?}\n",
        VERSION,
        terminal_safe(&report.project),
        report.stage
    );
    for f in &report.findings {
        show_finding(f, true);
    }
    println!(
        "\n{} inspected files · {:.2} MiB tracked working tree · {} unresolved required checks",
        report.snapshot.files.len(),
        report.snapshot.tracked_bytes as f64 / 1048576.0,
        rules::blocking(report)
    );
    if let Some(bytes) = report.snapshot.git_object_bytes {
        println!(
            "Local Git objects: {:.2} MiB (not remote GitHub/LFS storage)",
            bytes as f64 / 1048576.0
        );
    }
    for note in &report.snapshot.notes {
        println!("Scope: {note}");
    }
    println!(
        "\nUse `oracle next` for actions. Passing checks is not a certification of overall readiness."
    );
}

fn changes(format: Format, changes: Vec<String>, dry_run: bool) -> Result<()> {
    if format == Format::Json {
        json(
            &serde_json::json!({"schema_version":1,"oracle_version":VERSION,"dry_run":dry_run,"changes":changes}),
        )
    } else {
        if dry_run {
            println!("Preview only; no files changed.");
        }
        for c in changes {
            println!("{}", terminal_safe(&c));
        }
        Ok(())
    }
}

fn run(cli: &Cli) -> Result<u8> {
    // Informational commands do not need a project or a valid configuration.
    match &cli.command {
        Commands::Rules => {
            if cli.format == Format::Json {
                json(&serde_json::json!({"schema_version":1,"rules":rules::RULES}))?;
            } else {
                for rule in rules::RULES {
                    println!("{:<10} {:<10?} {}", rule.id, rule.starts_at, rule.title);
                }
            }
            return Ok(0);
        }
        Commands::Explain { id } => {
            let rule = rules::rule(id)?;
            if cli.format == Format::Json {
                json(rule)?;
            } else {
                println!(
                    "{} · {}\n\nWhy\n{}\n\nNext action\n{}\n\nDone when\n{}\n\nIntroduced at: {:?}\nSource: {}",
                    rule.id,
                    rule.title,
                    rule.why,
                    rule.action,
                    rule.done_when,
                    rule.starts_at,
                    rule.source
                );
            }
            return Ok(0);
        }
        _ => (),
    }
    let root = cli
        .root
        .canonicalize()
        .map_err(|_| "project directory does not exist or cannot be accessed")?;
    if !root.is_dir() {
        return Err("--root must point to a directory".into());
    }
    match &cli.command {
        Commands::Init {
            name,
            kind,
            stage,
            audience,
            dry_run,
        } => {
            let name = name.clone().unwrap_or_else(|| {
                root.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            });
            changes(
                cli.format,
                config::init(
                    &root,
                    &Config::new(name, *kind, *stage, *audience),
                    *dry_run,
                )?,
                *dry_run,
            )?;
        }
        Commands::Agent { command } => {
            let (client, remove, dry_run) = match command {
                AgentCommand::Install { client, dry_run } => (*client, false, *dry_run),
                AgentCommand::Remove { client, dry_run } => (*client, true, *dry_run),
            };
            changes(
                cli.format,
                agent::install(&root, client, remove, dry_run)?,
                dry_run,
            )?;
        }
        Commands::Evidence {
            command:
                EvidenceCommand::Record {
                    id,
                    result,
                    method,
                    summary,
                    artifact,
                },
        } => {
            let entry =
                evidence::record(&root, rules::rule(id)?, *result, *method, summary, artifact)?;
            if cli.format == Format::Json {
                json(&entry)?;
            } else {
                println!(
                    "Recorded {:?} attestation for {} with {} artifact(s).\nOracle verifies provenance/freshness; it did not execute or independently verify the scenario.",
                    entry.outcome,
                    entry.rule_id,
                    entry.artifacts.len()
                );
            }
        }
        Commands::Assess | Commands::Check { .. } | Commands::Next { .. } => {
            let config = config::load(&root)?;
            let report = rules::assess(&root, &config)?;
            if let Commands::Next { limit } = cli.command {
                let next = rules::next(&report, limit as usize);
                if cli.format == Format::Json {
                    json(
                        &serde_json::json!({"schema_version":1,"oracle_version":VERSION,"project":report.project,"stage":report.stage,"actions":next,"scope":report.snapshot.notes}),
                    )?;
                } else {
                    println!(
                        "Oracle · next actions for {}\n",
                        terminal_safe(&report.project)
                    );
                    if next.is_empty() {
                        println!(
                            "No unresolved active findings in the inspected scope. Deferred and unsupported work may remain."
                        );
                    }
                    for f in next {
                        show_finding(f, true);
                        println!("  Action: {}\n  Done when: {}\n", f.action, f.done_when);
                    }
                }
            } else {
                if cli.format == Format::Json {
                    json(&report)?;
                } else {
                    show_report(&report);
                }
                if matches!(cli.command, Commands::Check { strict: true })
                    && rules::blocking(&report) > 0
                {
                    return Ok(1);
                }
            }
        }
        Commands::Explain { .. } | Commands::Rules => unreachable!(),
    }
    Ok(0)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            if cli.format == Format::Json {
                eprintln!(
                    "{}",
                    serde_json::json!({"schema_version":1,"error":error.to_string()})
                );
            } else {
                eprintln!("oracle: {}", terminal_safe(&error.to_string()));
            }
            ExitCode::from(2)
        }
    }
}
