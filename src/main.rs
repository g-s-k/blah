use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex};

use futures::sync::mpsc;
use warp::{filters::ws, path, Filter, Future, Stream};

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

type Users = Arc<Mutex<HashMap<usize, mpsc::UnboundedSender<ws::Message>>>>;

fn connect_user(sock: ws::WebSocket, users: Users) -> impl Future<Item = (), Error = ()> {
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

    users.lock().unwrap().insert(my_id, tx);
    let users2 = users.clone();

    user_rx
        .for_each(move |msg| {
            user_message(my_id, msg, &users);
            Ok(())
        })
        .then(move |result| {
            user_disconnected(my_id, &users2);
            result
        })
        .map_err(move |e| {
            eprintln!("websocket error(uid={}): {}", my_id, e);
        })
}

fn user_message(my_id: usize, msg: ws::Message, users: &Users) {
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);

    for (&uid, tx) in users.lock().unwrap().iter() {
        if my_id != uid {
            match tx.unbounded_send(ws::Message::text(new_msg.clone())) {
                Ok(()) => (),
                Err(_disconnected) => {
                }
            }
        }
    }
}

fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);
    users.lock().unwrap().remove(&my_id);
}

fn main() {
    let users = Arc::new(Mutex::new(HashMap::new()));

    let router = path!("ws")
        .and(ws::ws2())
        .and(warp::any().map(move || users.clone()))
        .map(|wsck: ws::Ws2, users| wsck.on_upgrade(move |sock| connect_user(sock, users)))
        .or(warp::any().map(|| warp::reply::html(include_str!("static/index.html"))));

    warp::serve(router).run(SocketAddr::new("0.0.0.0".parse().unwrap(), 8080))
}
