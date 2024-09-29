use std::io;
use std::sync::Arc;
use std::time::Duration;

// Using parking_lot::Mutex for a timeout. We could alternatively use tokio::sync::Mutex, but
// the docs suggest only using it if absolutely necessary.
use parking_lot::Mutex;
use poem::endpoint::EmbeddedFilesEndpoint;
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::listener::{Acceptor, Listener, TcpListener};
use poem::middleware::AddData;
use poem::web::{Data, Json};
use poem::{handler, EndpointExt, Result, Route, Server};
use thiserror::Error;

use crate::DataInterface;
type DataInterfacePointer = Arc<Mutex<Box<dyn DataInterface>>>;
pub type PortsToTry = Box<dyn Iterator<Item = usize>>;

#[cfg(debug_assertions)]
#[derive(rust_embed::Embed)]
#[folder = "webapp/dist/"]
struct StaticFiles;

#[cfg(not(debug_assertions))]
#[derive(rust_embed::Embed)]
#[folder = "webapp/dist/"]
#[exclude = "*.map"]
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
    // TODO: Move these to DataReadError/DataSaveError
    // TODO: Add duration to the error message
    #[error(
        "Could not obtain lock to read data. It is likely already being saved or read.\nReloading \
         the page may help."
    )]
    FailedToObtainLockForReading,
    #[error(
        "Could not obtain lock to write data. It is likely already being saved or read.\nTry \
         again later."
    )]
    FailedToObtainLockForWriting,
    #[error("{0}")]
    FailedToSendExitSignal(tokio::sync::mpsc::error::SendError<ExitCode>),
}
impl ResponseError for ServerHTTPError {
    fn status(&self) -> StatusCode {
        // TODO: FailedToObtainLock could have a different error code
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

type ExitCodeSender = tokio::sync::mpsc::Sender<ExitCode>;

#[handler]
fn get_merge_data(
    Data(input): Data<&DataInterfacePointer>,
) -> Result<Json<crate::EntriesToCompare>> {
    // TODO: We can consider wrapping this IO in `tokio::spawn_blocking`, but it
    // doesn't seem crucial since there shouldn't actually be that much concurrency.
    // TODO: Is there some notion of a separate "lock for reading" that would allow
    // simultaneous reads?
    let timeout = Duration::from_millis(200);
    let input = input
        .try_lock_for(timeout)
        .ok_or(ServerHTTPError::FailedToObtainLockForReading)?;
    Ok(Json(input.scan().map_err(ServerHTTPError::from)?))
}

#[handler]
fn save(
    Data(input): Data<&DataInterfacePointer>,
    Json(data): Json<indexmap::IndexMap<String, String>>,
) -> Result<Json<()>> {
    // TODO: We can consider wrapping this IO in `tokio::spawn_blocking`, but it
    // doesn't seem crucial since there shouldn't actually be that much concurrency.
    let timeout = Duration::from_secs(3);
    let mut input = input
        .try_lock_for(timeout)
        .ok_or(ServerHTTPError::FailedToObtainLockForWriting)?;
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
    input: Box<dyn crate::DataInterface>,
    ports_to_try: PortsToTry,
    open_browser: bool,
) -> Result<(), MergeToolError> {
    let (terminate_channel, mut terminate_rx): (ExitCodeSender, _) = tokio::sync::mpsc::channel(10);
    let input: DataInterfacePointer = Arc::new(Mutex::new(input));
    let apis = Route::new()
        .at("/get_merge_data", poem::get(get_merge_data))
        .at("/save", poem::put(save))
        .at("/exit", poem::post(exit))
        .with(AddData::new(input))
        .with(AddData::new(terminate_channel));
    let app = Route::new()
        .nest("/", EmbeddedFilesEndpoint::<StaticFiles>::new())
        .nest("/api", apis);

    let acceptor = 'acceptor: {
        let mut error = None;
        for port in ports_to_try {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port));
            match listener.into_acceptor().await {
                Ok(a) => break 'acceptor a,
                Err(err) => {
                    eprintln!("Couldn't bind to port {port}.");
                    error = Some(err)
                }
            };
        }
        return Err(MergeToolError::FailedToOpenPort(error.unwrap()));
    };

    // Get the actual address we bound to. The primary reason to do this instead of
    // using `port` is to find out what port number the OS picked if we ended up
    // requesting port 0.
    let socket_addr = acceptor_to_socket_address(&acceptor)?;
    // Now that the acceptor exists, the browser should be able to connect.
    let http_address = format!("http://{socket_addr}");
    eprintln!("Listening at {http_address}.");
    if open_browser {
        tokio::task::spawn_blocking(move || {
            // Use `spawn_blocking` since `webbrowser::open` may block (for text-mode
            // browsers. TODO: find out if it blocks when running a fresh instance of
            // `firefox` on Linux.)
            eprintln!("Trying to launch a browser...");
            match open::that(http_address) {
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
                    // TODO: doesn't seem to work for dependents. This may be a
                    // good thing; see
                    // https://docs.rs/tokio/latest/tokio/signal/fn.ctrl_c.html#caveats.
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

/// Initialize tokio correctly and call `run_server`
pub fn run_server_sync(
    input: Box<dyn crate::DataInterface>,
    ports_to_try: PortsToTry,
    open_browser: bool,
) -> Result<(), MergeToolError> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run_server(input, ports_to_try, open_browser))
}
