use std::collections::HashMap;
use std::sync::mpsc::Sender;

pub type PlayerId = u64;

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub outgoing: Sender<String>,
}

#[derive(Debug, Default)]
pub struct World {
    next_player_id: PlayerId,
    players: HashMap<PlayerId, Player>,
}

impl World {
    pub fn join(&mut self, name: String, outgoing: Sender<String>) -> PlayerId {
        let id = self.next_player_id;
        self.next_player_id += 1;
        self.players.insert(id, Player { name, outgoing });
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

    pub fn broadcast(&mut self, message: &str) {
        self.players
            .retain(|_, player| player.outgoing.send(message.to_owned()).is_ok());
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
}
