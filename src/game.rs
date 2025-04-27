use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use tokio::sync::{mpsc, RwLock};
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
