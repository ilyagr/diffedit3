use std::net::ToSocketAddrs;

use log::info;
use serde::Serialize;
use warp::http::StatusCode;
use warp::Filter;
#[derive(rust_embed::RustEmbed)]
#[folder = "../webapp/dist"]
struct StaticFiles;

// See also https://github.com/informationsea/warp-embed/blob/master/examples/test-embedded.rs

fn result_to_warp_reply<T: Serialize, Err: Serialize>(result: Result<T, Err>) -> impl warp::Reply {
    match result {
        Ok(data) => warp::reply::with_status(warp::reply::json(&data), StatusCode::OK),
        Err(err) => {
            warp::reply::with_status(warp::reply::json(&err), StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() {
    let static_files = warp_embed::embed(&StaticFiles {});
    let server = warp::get().and(
        static_files.with(warp::log("http")).or(warp::path("api")
            .and(warp::path("get_merge_data"))
            .and(warp::path::end())
            .map(|| diff_tool_logic::Input::FakeData.scan())
            .map(result_to_warp_reply)),
    );
    let listen_to = "127.0.0.1:8080";
    eprintln!("Trying to listen at http://{listen_to}...");
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
}
