use clap::Parser;
use diffedit3_web::local_server::{run_server, MergeToolError};

/// Compare three directories in a browser and allow editing one of them
#[derive(Parser)]
#[command(version, about)]
pub struct LocalServerCli {
    #[command(flatten)]
    lib_cli: diffedit3_web::fs::Cli,
    /// Port to use for `http://127.0.0.1`
    #[arg(long, short, default_value = "8080")]
    port: usize,
    // TODO: Change syntax from `--port-range 8080 8085` to `--port 8080-8085`?
    /// Minimum and maximum port numbers to try for `http://127.0.0.1`.
    ///
    /// First, the minimum port is tried. If that is busy, the next port is
    /// tried, and so on.
    #[arg(long, num_args(2), conflicts_with = "port")]
    port_range: Option<Vec<usize>>,
    /// Do not try to open the browser automatically
    ///
    /// See https://crates.io/crates/open for a brief description of how the
    /// default browser is chosen. The `BROWSER` environment variable may be
    /// considered by `xdg-open` and similar commands.
    #[arg(long, short = 'N')]
    no_browser: bool,
    /// Make the server print debugging information
    #[arg(long, short)]
    verbose: bool,
}

fn exit_with_cli_error(s: String) -> ! {
    eprintln!("{s}");
    std::process::exit(2)
}

#[tokio::main]
async fn main() -> Result<(), MergeToolError> {
    let cli = LocalServerCli::parse();
    let input: diffedit3_web::fs::ThreeDirInput = match cli.lib_cli.try_into() {
        Ok(i) => i,
        Err(err) => {
            exit_with_cli_error(err.to_string());
        }
    };

    if cli.verbose {
        // TODO: We may consider deleting this or finding a way to get poem
        // to log each connection.
        if std::env::var_os("RUST_LOG").is_none() {
            std::env::set_var("RUST_LOG", "poem=debug");
        }
        tracing_subscriber::fmt::init();
    }

    let (min_port, max_port) = match cli.port_range {
        Some(v) => (v[0], v[1]), // Clap guarantees exactly two values
        None => (cli.port, cli.port),
    };
    if min_port > max_port {
        exit_with_cli_error(format!(
            "Error: the minimum port {min_port} cannot be greater than the maximum port \
             {max_port}."
        ));
    };
    if let Err(err) = run_server(input, min_port, max_port, !cli.no_browser).await {
        std::process::exit(match err {
            MergeToolError::IOError(err) => {
                eprintln!("{err}");
                3
            }
            MergeToolError::FailedToOpenPort(_) => {
                eprintln!("Failed to open HTTP port: {err}");
                3
            }
            MergeToolError::RequestedExitWithCode(code) => code,
            // TODO: Find somewhere to import the CtrlC exit code from.
            // Also, this may not be the correct thing to do on Windows.
            MergeToolError::CtrlC => 130,
        });
    };
    Ok(())
}
