use crate::world::Direction;

#[derive(Debug, PartialEq)]
pub enum Command {
    Go(Direction),
    Look,
    Say(String),
    Who,
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Empty,
    MissingMessage,
    MissingDirection,
    Unknown(String),
}

pub fn parse_command(input: &str) -> Result<Command, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::Empty);
    }

    let mut words = input.splitn(2, char::is_whitespace);
    let verb = words.next().expect("non-empty input has a first word");
    let rest = words.next().map(str::trim).filter(|text| !text.is_empty());

    match verb {
        "go" => {
            let Some(direction) = rest else {
                return Err(ParseError::MissingDirection);
            };
            let direction = match direction {
                "north" => Direction::North,
                "south" => Direction::South,
                "east" => Direction::East,
                "west" => Direction::West,
                other => return Err(ParseError::Unknown(other.to_owned())),
            };
            Ok(Command::Go(direction))
        }
        "look" => Ok(Command::Look),
        "who" => Ok(Command::Who),
        "quit" => Ok(Command::Quit),
        "say" => {
            let Some(message) = rest else {
                return Err(ParseError::MissingMessage);
            };
            Ok(Command::Say(message.to_owned()))
        }
        other => Err(ParseError::Unknown(other.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_message_with_spaces() {
        assert_eq!(
            parse_command("say hello from the gate"),
            Ok(Command::Say("hello from the gate".to_owned()))
        );
    }

    #[test]
    fn say_needs_a_message() {
        assert_eq!(parse_command("say"), Err(ParseError::MissingMessage));
    }
}
