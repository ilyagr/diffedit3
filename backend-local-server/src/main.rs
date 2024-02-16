use std::time::Duration;

use clap::Parser;
use poem::endpoint::EmbeddedFilesEndpoint;
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::listener::TcpListener;
use poem::middleware::AddData;
use poem::web::{Data, Json};
use poem::{handler, EndpointExt, Result, Route, Server};
use thiserror::Error;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;
// use serde::Serialize;

#[derive(rust_embed::RustEmbed)]
#[folder = "../webapp/dist"]
struct StaticFiles;

// https://github.com/pyrossh/rust-embed/blob/master/examples/poem.rs
// TODO: https://github.com/poem-web/poem/blob/master/examples/poem/embed-files/src/main.rs
// TODO: https://github.com/poem-web/poem/blob/master/examples/poem/custom-error/src/main.rs

// fn result_to_warp_reply<T: Serialize, Err: Serialize>(result: Result<T, Err>)
// -> impl warp::Reply {     match result {
//         Ok(data) => warp::reply::with_status(warp::reply::json(&data),
// StatusCode::OK),         Err(err) => {
//             warp::reply::with_status(warp::reply::json(&err),
// StatusCode::INTERNAL_SERVER_ERROR)         }
//     }
// }

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
    std::process::exit(code);
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = diff_tool_logic::Cli::parse();
    let input: diff_tool_logic::Input = cli
        .try_into()
        .unwrap_or_else(|err| panic!("{err}\nTODO: proper error instead of panic"));

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

    let (server_dead_send, mut server_dead_recv) = oneshot::channel();

    let listen_to = "127.0.0.1:8080";
    let http_address = format!("http://{listen_to}");
    let http_address_clone = http_address.clone();
    tokio::task::spawn_blocking(move || {
        // Try to avoid starting the browser if the server failed to start.
        std::thread::sleep(Duration::from_millis(200));
        match server_dead_recv.try_recv() {
            Ok(()) => { /* Server quit already */ }
            Err(TryRecvError::Empty) => {
                // TODO: Find a way to check whether the server started. Currently, if server
                // startup takes more than 200ms, the browser will launch.
                eprint!("Trying to launch a browser at {http_address_clone}...");
                match open::that(http_address_clone) {
                    Ok(_) => eprintln!(" Success!"),
                    Err(err) => eprintln!("\nFailed to launch a browser: {err}"),
                }
            }
            Err(TryRecvError::Closed) => { /* Should never happen */ }
        };
    });

    eprintln!("Trying to listen at {http_address}...");
    Server::new(TcpListener::bind(listen_to)).run(app).await?;
    let _ = server_dead_send.send(()); // No need to start the web browser. Don't care if the message failed to send,
                                       // though.
    Ok(())
}
