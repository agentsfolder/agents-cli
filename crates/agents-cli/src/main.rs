use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
enum Backend {
    VfsContainer,
    Materialize,
    VfsMount,
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

fn main() {
    let _cli = Cli::parse();

    // Placeholder until feat-loadag wires everything.
    println!("agents: not implemented yet");
}
