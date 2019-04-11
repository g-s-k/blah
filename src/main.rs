use std::net::SocketAddr;

use warp::{filters::ws, path, Filter, Future, Stream};

fn main() {
    let router = path!("ws")
        .and(ws::ws2())
        .map(|wsck: ws::Ws2| {
            wsck.on_upgrade(|sock| {
                let (tx, rx) = sock.split();
                rx.forward(tx).map(|_| ()).map_err(|e| {
                    eprintln!("websocket error: {:?}", e);
                })
            })
        })
        .or(warp::any().map(|| warp::reply::html(include_str!("static/index.html"))));

    warp::serve(router).run(SocketAddr::new("0.0.0.0".parse().unwrap(), 8080))
}
