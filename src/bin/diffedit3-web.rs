use clap::Parser;
use diffedit3::local_server::{run_server, MergeToolError};

/// Compare three directories in a browser and allow editing one of them
#[derive(Parser)]
#[command(version, about)]
pub struct LocalServerCli {
    #[command(flatten)]
    lib_cli: diffedit3::fs::Cli,
    /// Port to use for `http://127.0.0.1`
    #[arg(long, short, conflicts_with = "port_range")]
    port: Option<usize>,
    // TODO: Change syntax from `--port-range 8080 8085` to `--port 8080-8085`?
    /// Minimum and maximum port numbers to try for `http://127.0.0.1`.
    ///
    /// First, the minimum port is tried. If that is busy, the next port is
    /// tried, and so on.
    #[arg(long, num_args(2), default_values = ["8080", "8090"])]
    port_range: Vec<usize>,
    /// Do not try to open the browser automatically
    ///
    /// See <https://crates.io/crates/open> for a brief description of how the
    /// default browser is chosen. The `BROWSER` environment variable is not
    /// respected, unfortunately.
    // TODO(ilyagr): One way to respect the BROWSER environment variable might
    // be to use the `webbrowser` crate to get the browser command. It'd be
    // better to use something else to actually launch the browser, as
    // `webbrowser::open` has limited error handlind, like
    // `open::that_detached`.
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
    let input: Box<dyn diffedit3::DataInterface> = match cli.lib_cli.into_data_interface() {
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

    let (min_port, max_port) = match cli.port {
        Some(port) => (port, port),
        None => (cli.port_range[0], cli.port_range[1]), // Clap guarantees exactly two values
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
