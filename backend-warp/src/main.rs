use std::net::ToSocketAddrs;

use log::info;
use warp::Filter;
#[derive(rust_embed::RustEmbed)]
#[folder = "../webapp/dist"]
struct StaticFiles;

// See also https://github.com/informationsea/warp-embed/blob/master/examples/test-embedded.rs

#[tokio::main]
async fn main() {
    let static_files = warp_embed::embed(&StaticFiles {});
    // TODO: Handle rejection
    // https://github.com/seanmonstar/warp/blob/master/examples/rejections.rs
    let server = warp::get().and(
        static_files.with(warp::log("http")).or(warp::path("api")
            .and(warp::path("inputdata.json"))
            .and(warp::path::end())
            .map(|| warp::reply::json(&diff_tool_logic::Input::FakeData.scan().unwrap()))),
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
