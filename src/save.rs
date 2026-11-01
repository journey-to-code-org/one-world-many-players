use serde::{Deserialize, Serialize};

use crate::world::{PlayerId, RoomId};

#[derive(Debug, Deserialize, Serialize)]
pub struct SavedPlayer {
    pub id: PlayerId,
    pub name: String,
    pub room_id: RoomId,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SavedWorld {
    pub version: u32,
    pub players: Vec<SavedPlayer>,
}
