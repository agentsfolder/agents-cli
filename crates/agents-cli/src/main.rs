use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};

#[cfg(test)]
mod main_tests;

mod prevdf;
mod status;
mod syncer;
mod cleanup;
mod doctor;

#[derive(Debug, Clone, ValueEnum)]
enum Backend {
    VfsContainer,
    Materialize,
    VfsMount,
}

impl Backend {
    fn as_str(&self) -> &'static str {
        match self {
            Backend::VfsContainer => "vfs_container",
            Backend::Materialize => "materialize",
            Backend::VfsMount => "vfs_mount",
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "agents")]
#[command(about = "Project agent-native config from .agents/", long_about = None)]
struct Cli {
    /// Repository root (defaults to auto-discovery)
    #[arg(long)]
    repo: Option<PathBuf>,

    /// Emit machine-readable output (reserved)
    #[arg(long)]
    json: bool,

    /// Verbose output
    #[arg(short, long, global = true, conflicts_with = "quiet")]
    verbose: bool,

    /// Quiet output
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        #[arg(long)]
        preset: Option<String>,
    },
    Validate {
        #[arg(long)]
        profile: Option<String>,
    },
    Status,
    SetMode {
        mode: String,
        #[arg(long)]
        profile: Option<String>,
    },
    Preview {
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        backend: Option<Backend>,
        #[arg(long)]
        mode: Option<String>,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long, default_value_t = false)]
        keep_temp: bool,
    },
    Diff {
        #[arg(long)]
        agent: Option<String>,
        #[arg(long, default_value_t = false)]
        show: bool,
    },
    Sync {
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        backend: Option<Backend>,
    },
    Run {
        agent: String,
        #[arg(long)]
        mode: Option<String>,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long)]
        backend: Option<Backend>,
        #[arg(last = true, trailing_var_arg = true)]
        passthrough: Vec<String>,
    },
    Doctor {
        #[arg(long)]
        fix: bool,
        #[arg(long)]
        ci: bool,
    },
    Clean {
        #[arg(long)]
        agent: Option<String>,

        #[arg(long, default_value_t = false)]
        dry_run: bool,

        #[arg(long, default_value_t = false)]
        yes: bool,
    },
    Import {
        #[arg(long = "from")]
        from_agent: String,
        #[arg(long)]
        path: Option<PathBuf>,
    },
    Explain {
        path: PathBuf,
    },
    Compat,
    Test {
        #[command(subcommand)]
        command: TestCommands,
    },
}

#[derive(Debug, Subcommand)]
enum TestCommands {
    Adapters {
        #[arg(long)]
        agent: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Human,
    Json,
}

#[derive(Debug)]
struct AppContext {
    repo_root: PathBuf,
    output: OutputMode,
    verbose: bool,
    quiet: bool,
}

#[derive(Debug)]
pub(crate) enum ErrorCategory {
    NotInitialized,
    InvalidArgs,
    Io,
    SchemaInvalid,
    Conflict,
    PolicyDenied,
    ExternalToolMissing,
}

#[derive(Debug)]
pub(crate) struct AppError {
    category: ErrorCategory,
    message: String,
    context: Vec<String>,
}

impl AppError {
    fn not_initialized(repo_root: &Path) -> Self {
        AppError {
            category: ErrorCategory::NotInitialized,
            message: "repository is not initialized".to_string(),
            context: vec![
                format!(
                    "missing required file: {}",
                    repo_root.join(".agents/manifest.yaml").display()
                ),
                "hint: run `agents init`".to_string(),
            ],
        }
    }

    pub(crate) fn exit_code(&self) -> i32 {
        match self.category {
            ErrorCategory::InvalidArgs => 2,
            ErrorCategory::NotInitialized => 3,
            ErrorCategory::SchemaInvalid => 4,
            ErrorCategory::Io
            | ErrorCategory::Conflict
            | ErrorCategory::PolicyDenied
            | ErrorCategory::ExternalToolMissing => 5,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "error: {}", self.message)?;
        for line in &self.context {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

impl std::error::Error for AppError {}

type AppResult<T> = Result<T, AppError>;

fn dispatch(ctx: &AppContext, cmd: Commands) -> AppResult<()> {
    match cmd {
        Commands::Validate { .. } => cmd_validate(ctx),
        Commands::Status => cmd_status(ctx),

        Commands::Preview {
            agent,
            backend,
            mode,
            profile,
            keep_temp,
        } => {
            let agent = agent.unwrap_or_else(|| "core".to_string());
            let backend = backend.map(|b| match b {
                Backend::VfsContainer => agents_core::model::BackendKind::VfsContainer,
                Backend::Materialize => agents_core::model::BackendKind::Materialize,
                Backend::VfsMount => agents_core::model::BackendKind::VfsMount,
            });

            crate::prevdf::cmd_preview(
                &ctx.repo_root,
                crate::prevdf::PreviewOptions {
                    agent,
                    backend,
                    mode,
                    profile,
                    keep_temp,
                },
            )
        }

        Commands::Diff { agent, show } => {
            let agent = agent.unwrap_or_else(|| "core".to_string());
            crate::prevdf::cmd_diff(&ctx.repo_root, crate::prevdf::DiffOptions { agent, show })
        }

        Commands::Sync { agent, backend } => {
            let agent = agent.unwrap_or_else(|| "core".to_string());
            let backend = backend.map(|b| match b {
                Backend::VfsContainer => agents_core::model::BackendKind::VfsContainer,
                Backend::Materialize => agents_core::model::BackendKind::Materialize,
                Backend::VfsMount => agents_core::model::BackendKind::VfsMount,
            });

            crate::syncer::cmd_sync(
                &ctx.repo_root,
                crate::syncer::SyncOptions {
                    agent,
                    backend,
                    verbose: ctx.verbose,
                },
            )
        }

        Commands::Clean {
            agent,
            dry_run,
            yes,
        } => crate::cleanup::cmd_clean(
            &ctx.repo_root,
            crate::cleanup::CleanOptions {
                agent,
                dry_run,
                yes,
            },
        ),

        _ => Err(AppError::not_initialized(&ctx.repo_root)),
    }
}

fn cmd_status(ctx: &AppContext) -> AppResult<()> {
    crate::status::cmd_status(&ctx.repo_root, ctx.output)
}

fn cmd_validate(ctx: &AppContext) -> AppResult<()> {
    let opts = agents_core::loadag::LoaderOptions {
        require_schemas_dir: false,
    };

    match agents_core::loadag::load_repo_config(&ctx.repo_root, &opts) {
        Ok((_cfg, report)) => {
            if ctx.output == OutputMode::Human {
                if report.warnings.is_empty() {
                    println!("ok: .agents loaded");
                } else {
                    println!("ok: .agents loaded (with warnings)");
                    for w in report.warnings {
                        if let Some(path) = w.path {
                            eprintln!("warning: {}: {}", path.display(), w.message);
                        } else {
                            eprintln!("warning: {}", w.message);
                        }
                    }
                }
            }

            match agents_core::schemas::validate_repo(&ctx.repo_root) {
                Ok(()) => {
                    if ctx.output == OutputMode::Human {
                        println!("ok: schemas valid");
                    }
                    Ok(())
                }
                Err(err) => Err(AppError {
                    category: ErrorCategory::SchemaInvalid,
                    message: format!("schema invalid: {} ({})", err.path.display(), err.schema),
                    context: {
                        let mut c = vec![format!("pointer: {}", err.pointer), err.message];
                        if let Some(h) = err.hint {
                            c.push(h);
                        }
                        c
                    },
                }),
            }
        }
        Err(agents_core::loadag::LoadError::NotInitialized { .. }) => {
            Err(AppError::not_initialized(&ctx.repo_root))
        }
        Err(err) => Err(AppError {
            category: ErrorCategory::Io,
            message: err.to_string(),
            context: vec![],
        }),
    }
}

fn init_tracing(verbose: bool, quiet: bool) {
    let default_level = if quiet {
        "error"
    } else if verbose {
        "debug"
    } else {
        "info"
    };

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

fn main() {
    let cli = Cli::parse();
    init_tracing(cli.verbose, cli.quiet);

    let output = if cli.json {
        OutputMode::Json
    } else {
        OutputMode::Human
    };

    let repo_root = cli.repo.unwrap_or_else(|| std::env::current_dir().unwrap());

    let ctx = AppContext {
        repo_root,
        output,
        verbose: cli.verbose,
        quiet: cli.quiet,
    };

    match dispatch(&ctx, cli.command) {
        Ok(()) => std::process::exit(0),
        Err(err) => {
            eprint!("{err}");
            std::process::exit(err.exit_code());
        }
    }
}
