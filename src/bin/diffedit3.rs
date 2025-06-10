use clap::Parser;
use diffedit3::local_server::{MergeToolError, run_server};
use thiserror::Error;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

type PortRange = std::ops::RangeInclusive<usize>;

/// Compare three directories in a browser and allow editing one of them
#[derive(Parser)]
#[command(version, about)]
pub struct LocalServerCli {
    #[command(flatten)]
    lib_cli: diffedit3::fs::Cli,
    /// Port or port range to use for `http://127.0.0.1` (can be repeated)
    ///
    /// Port 0 is a special value that instructs the OS to pick a random unused
    /// port number. Ports 1-1023 are generally unavailable for use by
    /// unprivileged processes.
    ///
    /// For example, the default is equivalent to `--port 8080-8090 --port 0`.
    /// This means that port number 8080 is tried first. If that is busy, port
    /// 8081 is tried, and so on. If all ports between 8080 and 8090 (inclusive)
    /// are busy, port 0 is tried, meaning that the OS picks a random open port.
    #[arg(
        long, short, default_values = ["8080-8090", "0"],
        value_name = "PORT_OR_PORT_RANGE", value_parser = parse_port_range
    )]
    port: Vec<PortRange>,
    /// Do not try to open the browser automatically
    ///
    /// See <https://crates.io/crates/open> for a brief description of how the
    /// default browser is chosen. The `BROWSER` environment variable is not
    /// respected, unfortunately.
    // TODO(ilyagr): One way to respect the BROWSER environment variable might
    // be to use the `webbrowser` crate to get the browser command. It'd be
    // better to use something else to actually launch the browser, as
    // `webbrowser::open` has limited error handling, like
    // `open::that_detached`.
    #[arg(long, short = 'N')]
    no_browser: bool,
    /// Make the server print debugging information
    #[arg(long, short)]
    verbose: bool,
}

#[derive(Debug, PartialEq, Eq, Clone, Error)]
enum ParsePortRangeError {
    #[error(
        "A port range must be one unsigned integer or two unsigned integers separated by a -, for \
         example 1234 or 1234-56789"
    )]
    SyntaxError,
    #[error("The minimum port {0} cannot be greater than the maximum port {1}")]
    InequalityError(usize, usize),
}

fn parse_port_range(s: &str) -> Result<PortRange, ParsePortRangeError> {
    let parse_usize = |s: &str| {
        s.parse::<usize>()
            .map_err(|_| ParsePortRangeError::SyntaxError)
    };
    match s.split_once('-') {
        None => {
            let port = parse_usize(s)?;
            Ok(port..=port)
        }
        Some((first, second)) => {
            let (min_port, max_port) = (parse_usize(first)?, parse_usize(second)?);
            if min_port <= max_port {
                Ok(min_port..=max_port)
            } else {
                Err(ParsePortRangeError::InequalityError(min_port, max_port))
            }
        }
    }
}

fn exit_with_cli_error(s: String) -> ! {
    eprintln!("{s}");
    std::process::exit(2)
}

#[tokio::main]
async fn main() {
    let cli = LocalServerCli::parse();
    let input: Box<dyn diffedit3::DataInterface> = match cli.lib_cli.into_data_interface() {
        Ok(i) => i,
        Err(err) => {
            exit_with_cli_error(err.to_string());
        }
    };

    // Allow adjusting logging with `RUST_LOG` environment variable
    let mut tracing_filter = tracing_subscriber::EnvFilter::from_default_env();
    if cli.verbose {
        tracing_filter = tracing_filter.add_directive("poem=info".parse().unwrap());
    }
    tracing_subscriber::registry()
        .with(tracing_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    if let Err(err) = run_server(
        input,
        Box::new(cli.port.into_iter().flatten()),
        !cli.no_browser,
    )
    .await
    {
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
}
