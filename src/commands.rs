#[derive(Debug)]
pub enum Command {
    JoinGame,
    MakeMove {position:usize},
}

#[derive(Debug)]
pub enum ParseCommandError{
  InvalidCommand,
  InvalidParameters,
}

impl Command {
    pub fn from_str(s: &str) -> Result<Command, ParseCommandError> {
        let parts: Vec<&str> = s.split_whitespace().collect();
  
        if parts.is_empty() {
            return Err(ParseCommandError::InvalidCommand);
        }
  
        match parts[0] {
            "make_play" if parts.len() == 2 => {
                let position = parts[1].parse().map_err(|_| ParseCommandError::InvalidParameters)?;
                if position >= 1 && position <= 9 {
                    Ok(Command::MakeMove { position })
                } else {
                    Err(ParseCommandError::InvalidParameters)   
                }
            },
            "join_game" => {
                Ok(Command::JoinGame)
            },
            _ => Err(ParseCommandError::InvalidCommand),
        }
    }
  }
  