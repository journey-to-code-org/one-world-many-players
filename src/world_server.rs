use std::sync::mpsc::{self, Receiver, Sender};

use crate::protocol::Command;
use crate::world::{PlayerId, World};

pub enum WorldMessage {
    Join {
        name: String,
        outgoing: Sender<String>,
        reply: Sender<PlayerId>,
    },
    Command {
        player_id: PlayerId,
        command: Command,
    },
    Leave {
        player_id: PlayerId,
    },
}

pub fn channel() -> (Sender<WorldMessage>, Receiver<WorldMessage>) {
    mpsc::channel()
}

pub fn run(mut world: World, incoming: Receiver<WorldMessage>) {
    while let Ok(message) = incoming.recv() {
        match message {
            WorldMessage::Join {
                name,
                outgoing,
                reply,
            } => {
                let joined = name.clone();
                let id = world.join(name, outgoing);
                world.broadcast(&format!("{joined} joined the world."));
                let _ = reply.send(id);
            }
            WorldMessage::Command { player_id, command } => {
                world.handle(player_id, command);
            }
            WorldMessage::Leave { player_id } => {
                let name = world.player_name(player_id).map(str::to_owned);
                world.leave(player_id);
                if let Some(name) = name {
                    world.broadcast(&format!("{name} left the world."));
                }
            }
        }
    }
}
