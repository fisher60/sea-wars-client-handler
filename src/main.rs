use std::time::SystemTime;

use futures::SinkExt;
use futures::StreamExt;
use warp::filters::ws::Message;
use warp::Filter;
use warp::ws;


async fn handle_websocket_connection(websocket: warp::ws::WebSocket){
    println!("User connected!");
    //let (mut sender, mut receiver) = websocket.split();
    let (mut sender, _) = websocket.split();

    let tick_time_ms = 500;

    let mut last_tick_time = SystemTime::now();

    let tick_update_data = "some data";

    loop {
        let time_since_last_tick = SystemTime::now().duration_since(last_tick_time).expect("Time went backwards").as_secs() * 1000;
        if time_since_last_tick >= tick_time_ms {
            match sender.send(Message::text(tick_update_data)).await {
                Ok(_) => {
                    last_tick_time = SystemTime::now();
                },
                Err(_) => {
                    println!("Failed to send game update message, disconnecting websocket...");
                    break;
                }
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
