use futures::SinkExt;
use futures::StreamExt;
use warp::Filter;
use warp::ws;


async fn receive_message(websocket: warp::ws::WebSocket){
    let (mut sender, mut receiver) = websocket.split();

    while let Some(response) = receiver.next().await {
        let message = match response {
            Ok(msg) => msg,
            Err(_) => panic!("Recieved message caused an error"),
        };

        match sender.send(message).await {
            Ok(_) => continue,
            Err(_) => panic!("Error! Oh no!"),
        }
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
