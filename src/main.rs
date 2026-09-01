//! Binary entry point: argument handling, logging setup, and process exit
//! codes. The MCP server itself lives in the library crate.

use std::process::ExitCode;

use frontmatter_mcp::mcp::FrontmatterServer;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

/// Text printed for `--help`/`-h`.
const HELP: &str = "\
frontmatter-mcp - an MCP server that reads only the YAML frontmatter of
markdown documents, without loading the document body.

USAGE:
    frontmatter-mcp

    Run with no arguments to start the server, speaking MCP over stdio.

OPTIONS:
    -h, --help       Print this help text and exit
    -V, --version    Print the version and exit
";

/// What `main` should do, decided from the process arguments.
enum Action {
    /// Start the MCP server and serve requests over stdio.
    Serve,
    /// Print `--help` text and exit successfully.
    PrintHelp,
    /// Print `--version` text and exit successfully.
    PrintVersion,
    /// An argument was not recognised; exit with a non-zero status.
    Unknown(String),
}

/// Decides the action to take from the process's command-line arguments.
fn parse_args<I: Iterator<Item = String>>(mut args: I) -> Action {
    match args.next().as_deref() {
        Some("--help" | "-h") => Action::PrintHelp,
        Some("--version" | "-V") => Action::PrintVersion,
        Some(other) => Action::Unknown(other.to_owned()),
        None => Action::Serve,
    }
}

/// Initialises the `tracing` subscriber. All log output goes to stderr,
/// since stdout is reserved for MCP protocol frames. The level defaults to
/// `info` and can be overridden with `RUST_LOG`.
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();
}

/// Starts the MCP server on the stdio transport and runs until the client
/// disconnects.
///
/// # Errors
///
/// Returns an error if the transport fails to initialize or the service
/// terminates abnormally.
async fn serve() -> anyhow::Result<()> {
    let service = FrontmatterServer::new().serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

/// # Panics
///
/// Panics if writing to stdout fails (e.g. a broken pipe), the same as any
/// use of `print!`/`println!`.
#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned());
    match parse_args(args) {
        Action::PrintHelp => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        Action::PrintVersion => {
            println!("frontmatter-mcp {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Action::Unknown(arg) => {
            eprintln!("unknown argument: {arg}");
            ExitCode::from(2)
        }
        Action::Serve => {
            init_tracing();
            tracing::info!("frontmatter-mcp {} starting", env!("CARGO_PKG_VERSION"));
            match serve().await {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    tracing::error!("{error:#}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
