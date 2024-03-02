use std::io;
use std::sync::Arc;
use std::time::Duration;

use poem::endpoint::EmbeddedFilesEndpoint;
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::listener::{Acceptor, Listener, TcpListener};
use poem::middleware::AddData;
use poem::web::{Data, Json};
use poem::{handler, EndpointExt, Result, Route, Server};
use thiserror::Error;

use crate::DataInterface;

#[derive(rust_embed::RustEmbed)]
#[folder = "../webapp/dist"]
struct StaticFiles;

pub type ExitCode = i32;
#[derive(Debug, Error)]
pub enum MergeToolError {
    #[error("{0}")]
    IOError(#[from] std::io::Error),
    #[error("{0}")]
    FailedToOpenPort(std::io::Error),
    #[error("Client requested exit with error code {0}")]
    RequestedExitWithCode(ExitCode),
    #[error("Ctrl-C pressed")]
    CtrlC,
}

/// Errors that transform into error responses to HTTP requests
#[derive(Debug, Error)]
enum ServerHTTPError {
    #[error("{0}")]
    DataReadError(#[from] crate::DataReadError),
    #[error("{0}")]
    DataSaveError(#[from] crate::DataSaveError),
    #[error("{0}")]
    FailedToSendExitSignal(tokio::sync::mpsc::error::SendError<ExitCode>),
}
impl ResponseError for ServerHTTPError {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

type ExitCodeSender = tokio::sync::mpsc::Sender<ExitCode>;

#[handler]
fn get_merge_data(
    input: Data<&Arc<Box<dyn DataInterface>>>,
) -> Result<Json<crate::EntriesToCompare>> {
    // TODO: We can consider wrapping this IO in `tokio::spawn_blocking`, but it
    // doesn't seem crucial since there shouldn't actually be that much concurrency.
    // The could also be weird side effects.
    Ok(Json(input.scan().map_err(ServerHTTPError::from)?))
}

#[handler]
fn save(
    input: Data<&Arc<Box<dyn DataInterface>>>,
    Json(data): Json<indexmap::IndexMap<String, String>>,
) -> Result<Json<()>> {
    // TODO: We can consider wrapping this IO in `tokio::spawn_blocking`, but it
    // doesn't seem crucial since there shouldn't actually be that much concurrency.
    // The could also be weird side effects.
    input.save(data).map_err(ServerHTTPError::from)?;
    Ok(Json(()))
}

#[handler]
async fn exit(
    Json(code): Json<ExitCode>,
    Data(terminate_channel): Data<&ExitCodeSender>,
) -> Result<Json<()>> {
    eprintln!("Stopping the local server and exiting the diff editor with error code {code}.");
    terminate_channel
        .send(code)
        .await
        .map_err(ServerHTTPError::FailedToSendExitSignal)?;
    Ok(Json(()))
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

pub async fn run_server(
    input: impl crate::DataInterface,
    min_port: usize,
    max_port: usize,
    open_browser: bool,
) -> Result<(), MergeToolError> {
    let (terminate_channel, mut terminate_rx): (ExitCodeSender, _) = tokio::sync::mpsc::channel(10);
    let input: Arc<Box<dyn DataInterface>> = Arc::new(Box::new(input));
    let apis = Route::new()
        .at("/get_merge_data", poem::get(get_merge_data))
        .at("/save", poem::put(save))
        .at("/exit", poem::post(exit))
        .with(AddData::new(input))
        .with(AddData::new(terminate_channel));
    let app = Route::new()
        .nest("/", EmbeddedFilesEndpoint::<StaticFiles>::new())
        .nest("/api", apis);

    let mut port = min_port;
    let mut error = None;
    let acceptor = loop {
        if port > max_port {
            return Err(MergeToolError::FailedToOpenPort(error.unwrap()));
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
    eprintln!("Listening at {socket_addr}.");
    if open_browser {
        let http_address = format!("http://{socket_addr}");
        tokio::task::spawn_blocking(move || {
            // Use `spawn_blocking` since `webbrowser::open` may block (for text-mode
            // browsers. TODO: find out if it blocks when running a fresh instance of
            // `firefox` on Linux.)
            eprintln!("Trying to launch a browser at {http_address}...");
            match webbrowser::open(&http_address) {
                Ok(_) => eprintln!("Successfully launched browser."),
                Err(err) => eprintln!("Failed to launch a browser: {err}"),
            }
        });
    }

    let mut result = Ok(());
    Server::new_with_acceptor(acceptor)
        .run_with_graceful_shutdown(
            app,
            async {
                result = tokio::select! {
                    _ = tokio::signal::ctrl_c() => {
                        Err(MergeToolError::CtrlC)
                    },
                    Some(code) = terminate_rx.recv() => {
                        match code {
                            0 => Ok(()),
                            code => Err(MergeToolError::RequestedExitWithCode(code))
                        }
                    }
                }
            },
            Some(Duration::from_secs(5)),
        )
        .await?;
    result
}
