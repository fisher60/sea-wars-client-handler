use std::ptr::null;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use futures::SinkExt;
use futures::StreamExt;
use serde_json::json;
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

fn build_login_message(new_user_id: u32) -> Message {
    let payload = json!({
        "type": "login",
        "data": {
            "userId": new_user_id.to_string()
        },
        "error": null
    });
    Message::text(payload.to_string())
}

fn build_login_failure_message() -> Message {
    let payload = json!({
        "type": "login",
        "data": null,
        "error": "You must login before continuing"
    });
    Message::text(payload.to_string())
}

fn build_game_update_message() -> Message {
    let payload = json!({
        "type": "update",
        "data": {
            "state": "stuff"
        },
        "error": null
    });
    Message::text(payload.to_string())
}


async fn handle_websocket_connection(websocket: warp::ws::WebSocket, last_user_id: Arc<Mutex<u32>>){

    let mut new_user_id = 0;
    {
        let mut mutex_user_id = last_user_id.lock().expect("Could not lock user ID mutex.");
        *mutex_user_id += 1;
        new_user_id = *mutex_user_id;
    }

    println!("User connected! ID: {}", new_user_id.to_string());
    
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

    let last_user_id = Arc::new(Mutex::new(0));

    let routes = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(ws())
        .and(warp::any().map(move || last_user_id.clone()))
        .map(|ws: ws::Ws, last_user_id: Arc<Mutex<u32>>| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(move |socket| handle_websocket_connection(socket, last_user_id))
        });

    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let routes = index.or(routes);

    println!("Serving websocket at ws://127.0.0.1:8000/ws, demo client: http://127.0.0.1:8000");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}


static INDEX_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <title>Game</title>
    </head>
    <body>

        <p id="userId">UserId: Login to receive a user ID</p>

        <button onClick="login_button()">Login</button>

        <ul id="messageList">
        </ul>


        <script type="text/javascript">
            const uri = 'ws://' + location.host + '/ws';
            const ws = new WebSocket(uri);

            let messageCount = 0;

            const messagesUl = document.getElementById("messageList");
            const userId = document.getElementById("userId");

            function login_button() {
                ws.send("login");
            }

            function message(data) {
                const line = document.createElement('p');
                line.innerText = data;
                log.appendChild(line);
            }

            ws.onmessage = function(msg) {
                let newLi = document.createElement("li");

                console.log(msg);
                let parsedMessageData = JSON.parse(msg.data);
                let messageType = parsedMessageData.type;
                let messageData = parsedMessageData.data;

                if (messageType === "login") {
                    userId.innerHTML = `UserId: ${messageData.userId}`;
                }
                else if (messageType === "update") {
                    newLi.innerHTML = `MessageId: ${messageCount}</br>Type: ${messageType}</br>Data: ${messageData.state}`;
                    messageCount ++;
                    messagesUl.appendChild(newLi);
                }

                if (messagesUl.childElementCount > 3) {
                    messagesUl.removeChild(messagesUl.getElementsByTagName("li")[0]);
                };
            };
        </script>
    </body>
</html>
"#;
