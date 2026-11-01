use std::collections::HashMap;

use crate::entity::{Entity, Item, Npc};
use crate::protocol::Command;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlayerId(pub(crate) u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
            rooms: HashMap::from([(gate, gate_room), (road, road_room)]),
        }
    }
}

impl World {
    pub fn join(&mut self, name: String) -> PlayerId {
        let id = PlayerId(self.next_player_id);
        self.next_player_id += 1;
        self.players.insert(
            id,
            Player {
                name,
                room_id: RoomId(0),
            },
        );
        id
    }

    pub fn leave(&mut self, id: PlayerId) {
        self.players.remove(&id);
    }

    pub fn player_name(&self, id: PlayerId) -> Option<&str> {
        self.players.get(&id).map(|player| player.name.as_str())
    }

    pub fn player_names(&self) -> Vec<&str> {
        let mut names: Vec<_> = self
            .players
            .values()
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
            .values()
            .filter(|player| player.room_id == room_id)
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
}
