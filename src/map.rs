use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct Building {
    pub resource_name: String,
    pub upgrade_level: usize,
    pub base_upgrade_cost: usize,
    pub tick_money_base_value: usize,
}

pub struct MapGrid {
    pub x: isize,
    pub y: isize,
    pub building: Option<Building>,
}

impl MapGrid {
    pub fn to_string_coord(&self) -> String {
        format!("{}_{}", self.x, self.y)
    }
}

pub type GameMaps = Arc<RwLock<HashMap<Uuid, HashMap<String, MapGrid>>>>;
