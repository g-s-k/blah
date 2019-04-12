use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

use futures::sync::mpsc;
use hyper::Uri;
use serde_derive::*;
use warp::{filters::ws, path, Filter, Future, Stream};

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

struct Model {
    users: HashMap<usize, mpsc::UnboundedSender<ws::Message>>,
}

type ModelLink = Arc<RwLock<Model>>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BlahMsg {
    user_id: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    initial: bool,
}

fn connect_user(sock: ws::WebSocket, model: ModelLink) -> impl Future<Item = (), Error = ()> {
    let my_id = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    eprintln!("new chat user: {}", my_id);

    let (user_tx, user_rx) = sock.split();

    let (tx, rx) = mpsc::unbounded();
    warp::spawn(
        rx.map_err(|()| -> warp::Error { unreachable!("unbounded rx never errors") })
            .forward(user_tx)
            .map(|_tx_rx| ())
            .map_err(|ws_err| eprintln!("websocket send error: {}", ws_err)),
    );

    let new_msg = BlahMsg {
        user_id: my_id,
        text: None,
        initial: true,
    };
    let _ = tx.unbounded_send(ws::Message::text(
        serde_json::to_string(&new_msg).expect("could not serialize init message"),
    ));
    model.write().unwrap().users.insert(my_id, tx);

    let model2 = model.clone();
    user_rx
        .for_each(move |msg| {
            user_message(my_id, msg, &model);
            Ok(())
        })
        .then(move |result| {
            user_disconnected(my_id, &model2);
            result
        })
        .map_err(move |e| {
            eprintln!("websocket error(uid={}): {}", my_id, e);
        })
}

fn annotate_message(mut msg: &str) -> String {
    msg = msg.trim();

    if msg.parse::<Uri>().is_ok() {
        if msg.ends_with(".jpg") || msg.ends_with(".png") {
            return format!(r#"<img src="{}" alt="inline image" />"#, msg);
        }
    }

    msg.into()
}

fn user_message(my_id: usize, msg: ws::Message, model: &ModelLink) {
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = BlahMsg {
        user_id: my_id,
        text: Some(annotate_message(msg)),
        initial: false,
    };

    let msg_str = serde_json::to_string(&new_msg).expect("could not serialize message");

    for tx in model.read().unwrap().users.values() {
        let _ = tx.unbounded_send(ws::Message::text(msg_str.as_ref()));
    }
}

fn user_disconnected(my_id: usize, model: &ModelLink) {
    eprintln!("good bye user: {}", my_id);
    model.write().unwrap().users.remove(&my_id);
}

fn main() {
    let model = Arc::new(RwLock::new(Model {
        users: HashMap::new(),
        tmp_dir: TempDir::new().expect("Could not create temporary directory."),
    }));

    let router = path!("ws")
        .and(path::end())
        .and(ws::ws2())
        .and(warp::any().map(move || model.clone()))
        .map(|wsck: ws::Ws2, model| wsck.on_upgrade(move |sock| connect_user(sock, model)))
        .or(path!("blah.js").and(path::end()).map(|| {
            warp::reply::with_header(
                include_str!("static/blah.js"),
                "content-type",
                "text/javascript",
            )
        }))
        .or(path!("styles.css").and(path::end()).map(|| {
            warp::reply::with_header(
                include_str!("static/styles.css"),
                "content-type",
                "text/css",
            )
        }))
        .or(warp::any().map(|| warp::reply::html(include_str!("static/index.html"))));

    warp::serve(router).run(SocketAddr::new("0.0.0.0".parse().unwrap(), 8080))
}
