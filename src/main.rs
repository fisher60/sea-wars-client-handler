use std::time::Duration;

use futures::SinkExt;
use futures::StreamExt;
use tokio::time;
use warp::filters::ws::Message;
use warp::Filter;
use warp::ws;

const TICK_TIME_MS: Duration = Duration::from_millis(500);


async fn handle_websocket_connection(websocket: warp::ws::WebSocket){
    println!("User connected!");
    let (mut sender, _) = websocket.split();

    let tick_update_data = "some data";

    loop {
        time::sleep(TICK_TIME_MS).await;
        match sender.send(Message::text(tick_update_data)).await {
            Ok(_) => continue,
            Err(_) => {
                println!("Failed to send game update message, disconnecting websocket...");
                break;
            }
        }
    }

}


#[tokio::main]
async fn main() {
    println!("Setting up websocket server...");

    let routes = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(ws())
        .map(|ws: ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(move |socket| handle_websocket_connection(socket))
        });

    println!("Serving websocket at ws://127.0.0.1:8000/ws");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
