use std::sync::Arc;

use futures::SinkExt;
use futures::StreamExt;
use game::GameHandler;
use game::Player;
use game::build_login_failure_message;
use game::build_login_message;
use game::game_loop;
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::Filter;
use warp::filters::ws::Message;
use warp::filters::ws::WebSocket;
use warp::ws;

mod game;
mod map;

async fn user_connection(ws: WebSocket, game_handler: Arc<GameHandler>) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let client_id = Uuid::new_v4();

    println!("Client {} connected", client_id);

    let mut login_success = false;

    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error: {}", e);
                break;
            }
        };

        let str_msg = match msg.to_str() {
            Ok(str_msg) => str_msg,
            Err(e) => {
                eprintln!("Invalid login request: {:?}", e);
                continue;
            }
        };

        if str_msg == "login" {
            let resp_message = build_login_message(client_id.to_string());
            match ws_tx.send(resp_message).await {
                Ok(_) => {
                    login_success = true;
                    break;
                }
                Err(_) => {
                    eprintln!("Failed to send login message, disconnecting websocket...");
                    break;
                }
            }
        } else {
            match ws_tx.send(build_login_failure_message()).await {
                Ok(_) => continue,
                Err(_) => {
                    println!("Failed to send login fail message, disconnecting websocket...");
                    break;
                }
            }
        }
    }

    let player = Player {
        client_id,
        tx,
        money: 0,
    };

    if login_success {
        game_handler.register_client(client_id, player).await;

        // Task to send messages from server to client
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    }

    // Client disconnected
    game_handler.unregister_client(&client_id).await;
    println!("Client {} disconnected", client_id);
}

fn with_game_handler(
    game_handler: Arc<GameHandler>,
) -> impl Filter<Extract = (Arc<GameHandler>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || game_handler.clone())
}

#[tokio::main]
async fn main() {
    println!("Setting up websocket server...");

    let game_handler = Arc::new(GameHandler::new());

    tokio::spawn(game_loop(game_handler.clone()));

    let routes = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(ws())
        .and(with_game_handler(game_handler.clone()))
        .map(|ws: ws::Ws, game_handler: Arc<GameHandler>| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(move |socket| user_connection(socket, game_handler))
        });

    let index = warp::path::end().and(warp::fs::file("src/index.html"));
    let routes = index.or(routes);

    println!("Serving websocket at ws://127.0.0.1:8000/ws, demo client: http://127.0.0.1:8000");
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
