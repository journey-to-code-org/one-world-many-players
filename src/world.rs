use std::collections::HashMap;
use std::sync::mpsc::Sender;

use crate::entity::{Entity, Item, Npc};
use crate::protocol::Command;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlayerId(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RoomId(u64);

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
    pub outgoing: Sender<String>,
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
        let mut rooms = HashMap::new();
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
        rooms.insert(gate, gate_room);
        rooms.insert(
            road,
            Room {
                name: "Northern Road".into(),
                description: "The road runs back toward the gate.".into(),
                exits: HashMap::from([(Direction::South, gate)]),
                contents: Vec::new(),
            },
        );
        Self {
            next_player_id: 0,
            players: HashMap::new(),
            rooms,
        }
    }
}

impl World {
    pub fn join(&mut self, name: String, outgoing: Sender<String>) -> PlayerId {
        let id = PlayerId(self.next_player_id);
        self.next_player_id += 1;
        self.players.insert(
            id,
            Player {
                name,
                room_id: RoomId(0),
                outgoing,
            },
        );
        id
    }

    pub fn leave(&mut self, id: PlayerId) {
        self.players.remove(&id);
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

    pub fn player_name(&self, id: PlayerId) -> Option<&str> {
        self.players.get(&id).map(|player| player.name.as_str())
    }

    pub fn broadcast(&mut self, message: &str) {
        self.players
            .retain(|_, player| player.outgoing.send(message.to_owned()).is_ok());
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

    pub fn handle(&mut self, player_id: PlayerId, command: Command) {
        let Some(player) = self.players.get(&player_id) else {
            return;
        };
        let name = player.name.clone();
        let outgoing = player.outgoing.clone();

        match command {
            Command::Go(direction) => match self.move_player(player_id, direction) {
                Ok(room_id) => {
                    let room = &self.rooms[&room_id];
                    let _ = outgoing.send(format!("You enter {}.", room.name));
                }
                Err(error) => {
                    let _ = outgoing.send(format!("You cannot go that way: {error:?}"));
                }
            },
            Command::Look => {
                let room_id = self.players[&player_id].room_id;
                let room = &self.rooms[&room_id];
                let mut description = format!("{}\n{}", room.name, room.description);
                for entity in &room.contents {
                    description.push_str(&format!("\n- {}", entity.describe()));
                }
                let _ = outgoing.send(description);
            }
            Command::Say(message) => self.broadcast(&format!("{name} says: {message}")),
            Command::Who => {
                let _ = outgoing.send(format!("Online: {}", self.player_names().join(", ")));
            }
            Command::Quit => {
                let _ = outgoing.send("Goodbye".into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joined_players_share_one_sorted_list() {
        let mut world = World::default();
        let (sender, _receiver) = std::sync::mpsc::channel();
        world.join("Ryan".into(), sender.clone());
        world.join("Bryce".into(), sender);

        assert_eq!(world.player_names(), vec!["Bryce", "Ryan"]);
    }

    #[test]
    fn a_valid_exit_moves_the_player() {
        let mut world = World::default();
        let (sender, _receiver) = std::sync::mpsc::channel();
        let player = world.join("Bryce".into(), sender);

        assert_eq!(world.move_player(player, Direction::North), Ok(RoomId(1)));
    }
}
