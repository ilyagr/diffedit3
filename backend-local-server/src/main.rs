use std::net::ToSocketAddrs;

use clap::Parser;
use poem::endpoint::EmbeddedFilesEndpoint;
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::listener::TcpListener;
use poem::web::Json;
use poem::{handler, Result, Route, Server};
use serde::Serialize;
use thiserror::Error;
use tokio::runtime::EnterGuard;
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
fn get_merge_data() -> Result<Json<diff_tool_logic::EntriesToCompare>> {
    Ok(Json(
        diff_tool_logic::Input::FakeData
            .scan()
            .map_err(ServerError::from)?,
    ))
}
#[handler]
fn save(Json(data): Json<indexmap::IndexMap<String, String>>) -> Result<()> {
    diff_tool_logic::Input::FakeData
        .save(data)
        .map_err(ServerError::from)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let cli = diff_tool_logic::Cli::parse();
    let _input: diff_tool_logic::Input = cli
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
        .at("/save", poem::put(save));
    let app = Route::new()
        .nest("/", EmbeddedFilesEndpoint::<StaticFiles>::new())
        .nest("/api", apis);
    // .nest("/assets", EmbeddedFilesEndpoint::<StaticFiles>::new());
    let listen_to = "127.0.0.1:8080";
    eprintln!("Trying to listen at http://{listen_to}...");
    Server::new(TcpListener::bind(listen_to)).run(app).await

    /*
    let static_files = warp_embed::embed(&StaticFiles {});
    let api_paths = warp::path("get_merge_data").map(move || input.clone().scan());
    let api_paths = api_paths.and(warp::path::end()).map(result_to_warp_reply);
    let server = warp::path("api")
        .and(api_paths)
        .or(static_files.and(warp::get()))
        .with(warp::log("http"));
    let listen = listen_to.to_string().to_socket_addrs().unwrap();
    let binded: Vec<_> = listen
        .map(|x| {
            let binded = warp::serve(server.clone()).bind(x);
            info!("binded: {}", x);
            tokio::spawn(binded)
        })
        .collect();
    for one in binded {
        one.await.unwrap();
    }
    */
}
