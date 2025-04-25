use std::time::Duration;

use futures::SinkExt;
use futures::StreamExt;
use serde::Serialize;
use tokio::time;
use uuid::Uuid;
use warp::filters::ws::Message;
use warp::Filter;
use warp::ws;

const TICK_TIME_MS: Duration = Duration::from_millis(500);


#[derive(Debug, Serialize)]
struct Update {
    #[serde(skip_serializing_if = "Option::is_none")]
    money: Option<isize>,
}

#[derive(Debug, Serialize)]
struct Login {
    user_id: String,
}


#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
enum Response {
    Update(Update),
    Login(Login),
    Error(String),
}



fn build_login_message(new_user_id: String) -> Message {
    let json_resp = Response::Login(Login {user_id: new_user_id});
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}

fn build_login_failure_message() -> Message {
    let json_resp = Response::Error(String::from("You must login before continuing"));
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}

fn build_game_update_message() -> Message {
    let json_resp = Response::Update(Update { money: Some(0) });
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}


async fn handle_websocket_connection(websocket: warp::ws::WebSocket){

    let new_user_id = Uuid::new_v4().to_string();

    println!("User connected! ID: {}", new_user_id);
    
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
            let resp_message = build_login_message(new_user_id);
            match sender.send(resp_message).await {
                Ok(_) => break,
                Err(_) => {
                    println!("Failed to send login message, disconnecting websocket...");
                    break;
                }
            }
        }
        else {
            match sender.send(build_login_failure_message()).await {
                Ok(_) => continue,
                Err(_) => {
                    println!("Failed to send login fail message, disconnecting websocket...");
                    break;
                }
            }
        }
    }

    loop {
        time::sleep(TICK_TIME_MS).await;
        match sender.send(build_game_update_message()).await {
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

    let index = warp::path::end().and(warp::fs::file("src/index.html"));
    let routes = index.or(routes);

    println!("Serving websocket at ws://127.0.0.1:8000/ws, demo client: http://127.0.0.1:8000");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
