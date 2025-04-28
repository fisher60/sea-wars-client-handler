use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::Serialize;
use tokio::{
    sync::{RwLock, mpsc},
    time::interval,
};
use uuid::Uuid;
use warp::filters::ws::Message;

type Clients = Arc<RwLock<HashMap<Uuid, Player>>>;

pub struct Player {
    pub client_id: Uuid,
    pub money: isize,
    pub ws_tx: mpsc::UnboundedSender<Message>,
}

pub struct GameHandler {
    clients: Clients,
}

impl GameHandler {
    pub fn new() -> Self {
        GameHandler {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn broadcast(&self, msg: Message) {
        let clients = self.clients.read().await;
        for (client_uuid, player) in clients.iter() {
            if let Err(_disconnected) = player.ws_tx.send(msg.clone()) {
                println!(
                    "Failed to send game update message to client {}",
                    client_uuid
                );
            }
        }
    }

    pub async fn broadcast_game_update(&self) {
        let clients = self.clients.read().await;
        for (_, player) in clients.iter() {
            if let Err(_disconnected) = player.ws_tx.send(build_game_update_message(player)) {
                println!(
                    "Failed to send game update message to client {}",
                    player.client_id
                );
            }
        }
    }

    pub async fn send_to(&self, client_id: Uuid, msg: Message) {
        let clients = self.clients.read().await;
        if let Some(player) = clients.get(&client_id) {
            let _ = player.ws_tx.send(msg);
        }
    }

    pub async fn register_client(&self, client_id: Uuid, player: Player) {
        self.clients.write().await.insert(client_id, player);
    }

    pub async fn unregister_client(&self, client_id: &Uuid) {
        self.clients.write().await.remove(client_id);
    }

    pub async fn process_player_updates(&self) {
        let mut clients = self.clients.write().await;
        for (_, player) in clients.iter_mut() {
            player.money += 1;
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Update {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub money: Option<isize>,
}

#[derive(Debug, Serialize)]
pub struct Login {
    pub user_id: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum Response {
    Update(Update),
    Login(Login),
    Error(String),
}

pub fn build_login_message(new_user_id: String) -> Message {
    let json_resp = Response::Login(Login {
        user_id: new_user_id,
    });
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}

pub fn build_login_failure_message() -> Message {
    let json_resp = Response::Error(String::from("You must login before continuing"));
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}

pub fn build_game_update_message(player: &Player) -> Message {
    let json_resp = Response::Update(Update {
        money: Some(player.money),
    });
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}

pub async fn game_loop(game_handler: Arc<GameHandler>) {
    const TICK_TIME_MS: Duration = Duration::from_millis(500);

    let mut ticker = interval(TICK_TIME_MS);
    loop {
        game_handler.process_player_updates().await;

        game_handler.broadcast_game_update().await;
        ticker.tick().await;
    }
}
