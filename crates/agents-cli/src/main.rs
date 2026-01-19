use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};

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
    },
    Diff {
        #[arg(long)]
        agent: Option<String>,
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
enum ErrorCategory {
    NotInitialized,
    InvalidArgs,
    Io,
    SchemaInvalid,
    Conflict,
    PolicyDenied,
    ExternalToolMissing,
}

#[derive(Debug)]
struct AppError {
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

    fn exit_code(&self) -> i32 {
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

        // Until feat-loadag, everything else behaves like "not initialized".
        _ => Err(AppError::not_initialized(&ctx.repo_root)),
    }
}

fn cmd_validate(ctx: &AppContext) -> AppResult<()> {
    let manifest_path = ctx.repo_root.join(".agents/manifest.yaml");
    if !manifest_path.exists() {
        return Err(AppError::not_initialized(&ctx.repo_root));
    }

    // Placeholder until schema validation exists.
    if ctx.output == OutputMode::Human {
        println!("ok: .agents initialized");
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

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
