use std::collections::{HashMap, HashSet};
use std::io;

use serde::{Deserialize, Serialize};

use crate::entity::{Entity, Item, Npc};
use crate::protocol::Command;
use crate::save::{SavedPlayer, SavedWorld};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct PlayerId(pub(crate) u64);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RoomId(pub(crate) u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, PartialEq)]
pub enum MoveError {
    UnknownPlayer,
    NoExit(Direction),
}

pub struct Room {
    pub name: String,
    pub description: String,
    pub exits: HashMap<Direction, RoomId>,
    pub contents: Vec<Box<dyn Entity + Send>>,
}

pub struct Player {
    pub name: String,
    pub room_id: RoomId,
}

pub struct RoomView<'a> {
    pub room: &'a Room,
    pub players: Vec<&'a Player>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldEvent {
    pub recipient: Option<PlayerId>,
    pub message: String,
}

impl WorldEvent {
    pub fn for_all(message: impl Into<String>) -> Self {
        Self {
            recipient: None,
            message: message.into(),
        }
    }

    pub fn for_player(player_id: PlayerId, message: impl Into<String>) -> Self {
        Self {
            recipient: Some(player_id),
            message: message.into(),
        }
    }

    pub fn is_visible_to(&self, player_id: PlayerId) -> bool {
        self.recipient.is_none() || self.recipient == Some(player_id)
    }
}

pub struct World {
    next_player_id: u64,
    players: HashMap<PlayerId, Player>,
    online: HashSet<PlayerId>,
    rooms: HashMap<RoomId, Room>,
}

impl Default for World {
    fn default() -> Self {
        let gate = RoomId(0);
        let road = RoomId(1);
        let mut gate_room = Room {
            name: "South Gate".into(),
            description: "A weathered gate opens onto the northern road.".into(),
            exits: HashMap::from([(Direction::North, road)]),
            contents: Vec::new(),
        };
        gate_room
            .contents
            .push(Box::new(Item::new("Lantern", "A steady blue flame")));
        gate_room.contents.push(Box::new(Npc::new(
            "Mara",
            "The gatekeeper watches the northern road",
        )));
        let road_room = Room {
            name: "Northern Road".into(),
            description: "The road runs back toward the gate.".into(),
            exits: HashMap::from([(Direction::South, gate)]),
            contents: Vec::new(),
        };

        Self {
            next_player_id: 0,
            players: HashMap::new(),
            online: HashSet::new(),
            rooms: HashMap::from([(gate, gate_room), (road, road_room)]),
        }
    }
}

impl World {
    pub fn join(&mut self, name: String) -> PlayerId {
        if let Some((&id, _)) = self.players.iter().find(|(_, player)| player.name == name) {
            self.online.insert(id);
            return id;
        }
        let id = PlayerId(self.next_player_id);
        self.next_player_id += 1;
        self.players.insert(
            id,
            Player {
                name,
                room_id: RoomId(0),
            },
        );
        self.online.insert(id);
        id
    }

    pub fn disconnect(&mut self, id: PlayerId) {
        self.online.remove(&id);
    }

    pub fn player_name(&self, id: PlayerId) -> Option<&str> {
        self.players.get(&id).map(|player| player.name.as_str())
    }

    pub fn player_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self
            .online
            .iter()
            .filter_map(|id| self.players.get(id))
            .map(|player| player.name.as_str())
            .collect();
        names.sort_unstable();
        names
    }

    pub fn move_player(
        &mut self,
        player_id: PlayerId,
        direction: Direction,
    ) -> Result<RoomId, MoveError> {
        let current_room = self
            .players
            .get(&player_id)
            .ok_or(MoveError::UnknownPlayer)?
            .room_id;
        let destination = self
            .rooms
            .get(&current_room)
            .and_then(|room| room.exits.get(&direction))
            .copied()
            .ok_or(MoveError::NoExit(direction))?;
        self.players
            .get_mut(&player_id)
            .expect("player was checked above")
            .room_id = destination;
        Ok(destination)
    }

    pub fn room_view(&self, room_id: RoomId) -> Option<RoomView<'_>> {
        let room = self.rooms.get(&room_id)?;
        let players = self
            .players
            .iter()
            .filter(|(id, player)| self.online.contains(id) && player.room_id == room_id)
            .map(|(_, player)| player)
            .collect();
        Some(RoomView { room, players })
    }

    pub fn handle(&mut self, player_id: PlayerId, command: Command) -> Vec<WorldEvent> {
        let Some(player) = self.players.get(&player_id) else {
            return Vec::new();
        };
        let name = player.name.clone();

        let event = match command {
            Command::Go(direction) => match self.move_player(player_id, direction) {
                Ok(room_id) => WorldEvent::for_player(
                    player_id,
                    format!("You enter {}.", self.rooms[&room_id].name),
                ),
                Err(error) => {
                    WorldEvent::for_player(player_id, format!("You cannot go that way: {error:?}"))
                }
            },
            Command::Look => {
                let room_id = self.players[&player_id].room_id;
                WorldEvent::for_player(
                    player_id,
                    render_room(self.room_view(room_id).expect("player room exists")),
                )
            }
            Command::Say(message) => WorldEvent::for_all(format!("{name} says: {message}")),
            Command::Who => WorldEvent::for_player(
                player_id,
                format!("Online: {}", self.player_names().join(", ")),
            ),
            Command::Quit => WorldEvent::for_player(player_id, "Goodbye"),
        };
        vec![event]
    }

    pub fn snapshot(&self) -> SavedWorld {
        let mut players: Vec<_> = self
            .players
            .iter()
            .map(|(id, player)| SavedPlayer {
                id: *id,
                name: player.name.clone(),
                room_id: player.room_id,
            })
            .collect();
        players.sort_by_key(|player| player.id);
        SavedWorld {
            version: 1,
            players,
        }
    }

    pub fn from_snapshot(saved: SavedWorld) -> io::Result<Self> {
        if saved.version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unsupported world version {}", saved.version),
            ));
        }
        let mut world = Self::default();
        for player in saved.players {
            world.next_player_id = world.next_player_id.max(player.id.0.saturating_add(1));
            if !world.rooms.contains_key(&player.room_id) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "saved player references an unknown room",
                ));
            }
            world.players.insert(
                player.id,
                Player {
                    name: player.name,
                    room_id: player.room_id,
                },
            );
        }
        Ok(world)
    }
}

pub fn render_room(view: RoomView<'_>) -> String {
    use std::fmt::Write;

    let mut message = format!("{}\n{}", view.room.name, view.room.description);
    for entity in &view.room.contents {
        let _ = write!(message, "\n- {}", entity.describe());
    }
    if !view.players.is_empty() {
        message.push_str("\nPlayers:");
        for player in view.players {
            let _ = write!(message, "\n- {}", player.name);
        }
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joined_players_share_one_sorted_list() {
        let mut world = World::default();
        world.join("Ryan".into());
        world.join("Bryce".into());
        assert_eq!(world.player_names(), vec!["Bryce", "Ryan"]);
    }

    #[test]
    fn a_valid_exit_moves_the_player() {
        let mut world = World::default();
        let player = world.join("Bryce".into());
        assert_eq!(world.move_player(player, Direction::North), Ok(RoomId(1)));
    }

    #[test]
    fn a_snapshot_restores_player_positions_without_presence() {
        let mut world = World::default();
        let player = world.join("Bryce".into());
        world.move_player(player, Direction::North).unwrap();

        let restored = World::from_snapshot(world.snapshot()).unwrap();

        assert!(restored.player_names().is_empty());
        assert_eq!(restored.players[&player].room_id, RoomId(1));
    }
}
