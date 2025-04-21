use std::time::Duration;

use futures::SinkExt;
use futures::StreamExt;
use tokio::time;
use warp::filters::ws::Message;
use warp::Filter;
use warp::ws;

const TICK_TIME_MS: Duration = Duration::from_millis(500);

#[derive(Debug, PartialEq)]
enum MessageType {
    GameStateUpdate,
    Login,
}

impl MessageType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "update" => Some(MessageType::GameStateUpdate),
            "login" => Some(MessageType::Login),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            MessageType::GameStateUpdate => "update",
            MessageType::Login => "login",
        }
    }
}

#[derive(Debug)]
struct Response {
    message_type: MessageType,
}

impl Response {
    fn new(message_type: MessageType) -> Self {
        Response { message_type }
    }
}


async fn handle_websocket_connection(websocket: warp::ws::WebSocket){
    println!("User connected!");
    let (mut sender, mut recv) = websocket.split();

    while let Some(result) = recv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error: {}", e);
                return;
            }
        };

        let str_msg = match msg.to_str() {
            Ok(str_msg) => str_msg,
            Err(e) => {
                eprintln!("Invalid login request: {:?}", e);
                return;
            }
        };

        if str_msg == "login" {
            let resp_message = Message::text("login success!");
            match sender.send(resp_message).await {
                Ok(_) => break,
                Err(_) => {
                    println!("Failed to send login message, disconnecting websocket...");
                    break;
                }
            }
        }
        else {
            let resp_message = Message::text("You must login before continuing...");
            match sender.send(resp_message).await {
                Ok(_) => continue,
                Err(_) => {
                    println!("Failed to send login fail message, disconnecting websocket...");
                    break;
                }
            }
        }
    }

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
