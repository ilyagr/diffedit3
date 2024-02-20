use std::io;

use clap::Parser;
use poem::endpoint::EmbeddedFilesEndpoint;
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::listener::{Acceptor, Listener, TcpListener};
use poem::middleware::AddData;
use poem::web::{Data, Json};
use poem::{handler, EndpointExt, Result, Route, Server};
use thiserror::Error;

#[derive(rust_embed::RustEmbed)]
#[folder = "../webapp/dist"]
struct StaticFiles;

/// Compare three directories in a browser and allow editing one of them
#[derive(Parser)]
#[command(version, about)]
pub struct LocalServerCli {
    #[command(flatten)]
    lib_cli: diff_tool_logic::Cli,
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
}

#[derive(Debug, Error)]
enum ServerError {
    #[error("{0}")]
    DataReadError(#[from] diff_tool_logic::DataReadError),
    #[error("{0}")]
    DataSaveError(#[from] diff_tool_logic::DataSaveError),
}
impl ResponseError for ServerError {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[handler]
fn get_merge_data(
    input: Data<&diff_tool_logic::Input>,
) -> Result<Json<diff_tool_logic::EntriesToCompare>> {
    Ok(Json(input.scan().map_err(ServerError::from)?))
}
#[handler]
fn save(
    input: Data<&diff_tool_logic::Input>,
    Json(data): Json<indexmap::IndexMap<String, String>>,
) -> Result<Json<()>> {
    input.save(data).map_err(ServerError::from)?;
    Ok(Json(()))
}
#[handler]
fn exit(Json(code): Json<i32>) -> Result<Json<()>> {
    eprintln!("Stopping the local server and exiting the diff editor with error code {code}.");
    std::process::exit(code);
}

fn acceptor_to_socket_address(
    acceptor: &poem::listener::TcpAcceptor,
) -> Result<std::net::SocketAddr, io::Error> {
    match acceptor
        .local_addr()
        .into_iter()
        .filter_map(|addr| addr.as_socket_addr().cloned())
        .next()
    {
        Some(addr) => Ok(addr),
        None => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(
                "Error: Unexpectedly listening at something other than a socket address or at no \
                 address: {:?}",
                &acceptor.local_addr()
            ),
        )),
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = LocalServerCli::parse();
    let input: diff_tool_logic::Input = cli.lib_cli.try_into().unwrap_or_else(|err| {
        eprintln!("Error: {err}");
        std::process::exit(2)
    });

    /* Taken from the example. What's this for?

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();
    */

    let apis = Route::new()
        .at("/get_merge_data", poem::get(get_merge_data))
        .at("/save", poem::put(save))
        .at("/exit", poem::post(exit))
        .with(AddData::new(input));
    let app = Route::new()
        .nest("/", EmbeddedFilesEndpoint::<StaticFiles>::new())
        .nest("/api", apis);

    let (min_port, max_port) = match cli.port_range {
        Some(v) => (v[0], v[1]), // Clap guarantees exactly two values
        None => (cli.port, cli.port),
    };
    if min_port > max_port {
        eprintln!(
            "Error: the minimum port {min_port} cannot be greater than the maximum port \
             {max_port}."
        );
        std::process::exit(2)
    };
    let mut port = min_port;
    let mut error = None;
    let acceptor = loop {
        if port > max_port {
            return Err(error.unwrap());
        }
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port));
        match listener.into_acceptor().await {
            Ok(a) => break a,
            Err(err) => {
                eprintln!("Couldn't bind to port {port}.");
                error = Some(err)
            }
        };
        port += 1;
    };

    // Get the actual address we bound to. The primary reason to do this instead of
    // using `port` is to find out what port number the OS picked if `cli.port==0`.
    let socket_addr = acceptor_to_socket_address(&acceptor)?;
    // Now that the acceptor exists, the browser should be able to connect IIUC.
    let http_address = format!("http://{socket_addr}");
    eprintln!("Listening at {http_address}.");
    if !cli.no_browser {
        eprint!("Trying to launch a browser at {http_address}...");
        match open::that(&http_address) {
            Ok(_) => eprintln!(" Success!"),
            Err(err) => eprintln!("\nFailed to launch a browser: {err}"),
        }
    }

    Server::new_with_acceptor(acceptor).run(app).await?;
    Ok(())
}
