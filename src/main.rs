mod message;

use futures_util::{SinkExt, StreamExt};
use message::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use warp::Filter;

#[tokio::main]
async fn main() {
    let rooms: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let rooms_filter = warp::any().map(move || Arc::clone(&rooms));

    let send_message = warp::path!("send" / String)
        .and(warp::post())
        .and(warp::body::json())
        .and(rooms_filter.clone())
        .and_then(
            |room: String,
             msg: Message,
             rooms: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>| async move {
                let rooms = rooms.read().await;
                if let Some(tx) = rooms.get(&room) {
                    tx.send(msg).unwrap();
                }
                Ok::<_, warp::Rejection>(warp::reply())
            },
        );

    let receive_messages = warp::path!("receive" / String)
        .and(warp::ws())
        .and(rooms_filter.clone())
        .map(
            |room: String,
             ws: warp::ws::Ws,
             rooms: Arc<RwLock<HashMap<String, broadcast::Sender<Message>>>>| {
                ws.on_upgrade(move |socket| {
                    let rooms = Arc::clone(&rooms);
                    async move {
                        let mut rx = {
                            let mut rooms = rooms.write().await;
                            if let Some(tx) = rooms.get(&room) {
                                tx.subscribe()
                            } else {
                                let (tx, rx) = broadcast::channel(100);
                                rooms.insert(room.clone(), tx);
                                rx
                            }
                        };
                        let (mut user_ws_tx, mut user_ws_rx) = socket.split();
                        let tx_task = tokio::spawn(async move {
                            while let Ok(msg) = rx.recv().await {
                                let msg =
                                    warp::ws::Message::text(serde_json::to_string(&msg).unwrap());
                                user_ws_tx.send(msg).await.unwrap();
                            }
                        });
                        let rx_task = tokio::spawn(async move {
                            while let Some(result) = user_ws_rx.next().await {
                                let _ = result.unwrap();
                            }
                        });
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
