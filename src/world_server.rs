use std::sync::mpsc::{self, Receiver, Sender};

use tokio::sync::broadcast;

use crate::protocol::Command;
use crate::world::{PlayerId, World, WorldEvent};

pub enum WorldMessage {
    Join {
        name: String,
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

pub fn run(
    mut world: World,
    incoming: Receiver<WorldMessage>,
    world_events: broadcast::Sender<WorldEvent>,
) {
    while let Ok(message) = incoming.recv() {
        match message {
            WorldMessage::Join { name, reply } => {
                let joined = name.clone();
                let id = world.join(name);
                let _ =
                    world_events.send(WorldEvent::for_all(format!("{joined} joined the world.")));
                let _ = reply.send(id);
            }
            WorldMessage::Command { player_id, command } => {
                for event in world.handle(player_id, command) {
                    let _ = world_events.send(event);
                }
            }
            WorldMessage::Leave { player_id } => {
                let name = world.player_name(player_id).map(str::to_owned);
                world.leave(player_id);
                if let Some(name) = name {
                    let _ =
                        world_events.send(WorldEvent::for_all(format!("{name} left the world.")));
                }
            }
        }
    }
}
