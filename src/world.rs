use std::collections::HashMap;

pub type PlayerId = u64;

#[derive(Debug)]
pub struct Player {
    pub name: String,
}

#[derive(Debug, Default)]
pub struct World {
    next_player_id: PlayerId,
    players: HashMap<PlayerId, Player>,
}

impl World {
    pub fn join(&mut self, name: String) -> PlayerId {
        let id = self.next_player_id;
        self.next_player_id += 1;
        self.players.insert(id, Player { name });
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
}
