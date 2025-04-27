use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::Serialize;
use tokio::{sync::{mpsc, RwLock}, time::interval};
use uuid::Uuid;
use warp::filters::ws::Message;

type Clients = Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<Message>>>>;

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
        for (client_uuid, tx) in clients.iter() {
            if let Err(_disconnected) = tx.send(msg.clone()) {
                println!("Failed to send game update message to client {}, disconnecting websocket...", client_uuid);
            }
        }
    }

    pub async fn send_to(&self, client_id: Uuid, msg: Message) {
        let clients = self.clients.read().await;
        if let Some(tx) = clients.get(&client_id) {
            let _ = tx.send(msg);
        }
    }

    pub async fn register_client(&self, client_id: Uuid, sender: mpsc::UnboundedSender<Message>) {
        self.clients.write().await.insert(client_id, sender);
    }

    pub async fn unregister_client(&self, client_id: &Uuid) {
        self.clients.write().await.remove(client_id);
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
    let json_resp = Response::Login(Login {user_id: new_user_id});
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}

pub fn build_login_failure_message() -> Message {
    let json_resp = Response::Error(String::from("You must login before continuing"));
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}

pub fn build_game_update_message() -> Message {
    let json_resp = Response::Update(Update { money: Some(0) });
    return Message::text(serde_json::to_string(&json_resp).unwrap());
}

pub async fn game_loop(game_handler: Arc<GameHandler>) {
    const TICK_TIME_MS: Duration = Duration::from_millis(500);
    
    let mut ticker = interval(TICK_TIME_MS);
    loop {
        ticker.tick().await;
        game_handler.broadcast(build_game_update_message()).await;
    }
}
