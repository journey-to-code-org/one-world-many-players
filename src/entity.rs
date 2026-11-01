pub trait Entity {
    fn name(&self) -> &str;
    fn description(&self) -> &str;

    fn describe(&self) -> String {
        format!("{}: {}", self.name(), self.description())
    }
}

pub struct Item {
    name: String,
    description: String,
}

impl Item {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

impl Entity for Item {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

pub struct Npc {
    name: String,
    job: String,
}

impl Npc {
    pub fn new(name: impl Into<String>, job: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            job: job.into(),
        }
    }
}

impl Entity for Npc {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.job
    }
}

pub fn description_line<T>(entity: &T) -> String
where
    T: Entity,
{
    format!("- {}", entity.describe())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_types_share_the_entity_contract() {
        let lantern = Item::new("Lantern", "A steady blue flame");
        let gatekeeper = Npc::new("Mara", "Gatekeeper");

        assert_eq!(description_line(&lantern), "- Lantern: A steady blue flame");
        assert_eq!(description_line(&gatekeeper), "- Mara: Gatekeeper");
    }
}
