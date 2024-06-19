mod message;

use futures_util::{SinkExt, StreamExt};
use message::Message;
use tokio::sync::broadcast;
use warp::Filter;

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

    let receive_messages = warp::path("receive").and(warp::ws()).and(tx.clone()).map(
        |ws: warp::ws::Ws, tx: broadcast::Sender<Message>| {
            ws.on_upgrade(move |socket| {
                let mut rx = tx.subscribe();
                let (mut user_ws_tx, mut user_ws_rx) = socket.split();
                let tx_task = tokio::spawn(async move {
                    while let Ok(msg) = rx.recv().await {
                        let msg = warp::ws::Message::text(serde_json::to_string(&msg).unwrap());
                        user_ws_tx.send(msg).await.unwrap();
                    }
                });
                let rx_task = tokio::spawn(async move {
                    while let Some(result) = user_ws_rx.next().await {
                        let _ = result.unwrap();
                    }
                });
                async move {
                    tx_task.await.unwrap();
                    rx_task.await.unwrap();
                }
            })
        },
    );

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["Content-Type"])
        .allow_methods(vec!["POST", "GET"]);
    let routes = send_message.or(receive_messages).with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
