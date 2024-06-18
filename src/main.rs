mod message;

use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use warp::Filter;
use message::Message;

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(100);
    let tx = warp::any().map(move || tx.clone());

    let send_message = warp::path("send")
        .and(warp::post())
        .and(warp::body::json())
        .and(tx.clone())
        .map(|msg: Message, tx: broadcast::Sender<Message>| {
            tx.send(msg).unwrap();
            warp::reply()
        });

    let receive_messages = warp::path("receive")
        .and(warp::ws())
        .and(tx.clone())
        .map(|ws: warp::ws::Ws, tx: broadcast::Sender<Message>| {
            ws.on_upgrade(move |socket| {
                let mut rx = tx.subscribe();
                let (user_ws_tx, mut user_ws_rx) = socket.split();
                tokio::spawn(async move {
                    while let Ok(msg) = rx.recv().await {
                        let msg = warp::ws::Message::text(serde_json::to_string(&msg).unwrap());
                        user_ws_tx.send(msg).await.unwrap();
                    }
                });
                tokio::spawn(async move {
                    while let Some(result) = user_ws_rx.next().await {
                        let _ = result.unwrap();
                    }
                });
            })
        });

    let routes = send_message.or(receive_messages);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}