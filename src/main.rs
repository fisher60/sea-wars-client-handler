// https://github.com/altonotch/rust-warp-websockets-example/blob/master/src/main.rs

use futures::SinkExt;
use futures::StreamExt;
use warp::Filter;
use warp::ws;
use warp::filters::ws::Message;

// async fn send_message(mut websocket: warp::ws::WebSocket){
//     let msg = Message::text("hello_world");
//     if let Err(e) = websocket.start_send_unpin(msg){
//         eprintln!("An error occured: {:?}", e)
//     };
// }

async fn receive_message(websocket: warp::ws::WebSocket){

    let (mut sender, mut receiver) = websocket.split();

    while let Some(body) = receiver.next().await {
        let msg_to_client = Message::text("hello_world");
        let _message = match body {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error reading message on websocket: {}", e);
                break;
            }
        };

        if let Err(e) = sender.start_send_unpin(msg_to_client){
            eprintln!("Could not send message to client {:?}", e)
        };
    }

}

#[tokio::main]
async fn main() {

    let routes = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(ws())
        .map(|ws: ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(receive_message)
        });

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
