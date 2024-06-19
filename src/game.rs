use std::collections::HashMap;

pub enum GameState {
  Waiting,
  Playing {current_turn: char},
  End {winner: char}
}

pub struct Game {
  board: [[char; 3];3],
  state: GameState,
  players: HashMap<String, char>
}

impl Game {
  pub fn new() -> Self {
    Game {
      board: [[' '; 3]; 3],
      state: GameState::Waiting,
      players: HashMap::new()
    }
  }

  pub fn add_player(&mut self, player_id: String) -> Result<char, &str> {
    if self.players.len() >= 2 {
      return Err("O jogo está completo");
    }

    let symbol = if self.players.len() == 0 {'X'} else {'O'};
    self.players.insert(player_id, symbol);
    Ok(symbol)
  }

  pub fn remove_player(&mut self, player_id: &str) -> Result<(), &str> {
    if self.players.remove(player_id).is_some() {
      if self.players.len() == 1 {
        let remaining_player_id = self.players.keys().next().unwrap();

        self.players.insert(remaining_player_id.clone(), 'X');
        self.reset_board();
      }   

      Ok(())
    } else {
      Err("Player não encontrado")
    }
  }

  fn reset_board(&mut self) {
    self.board = [[' '; 3]; 3];
  }

  pub fn make_move(&mut self, player_id: &str, row: usize, col: usize) -> Result<(), &str> {
    if let GameState::Playing {current_turn} = self.state {
      if self.board[row][col] != ' ' {
        return Err("Esta célula já foi preenchida")
      }

      if let Some(&symbol) = self.players.get(player_id) {
        if symbol != current_turn {
          return Err("Não é sua vez de jogar")
        }

        self.board[row][col] = symbol; 
        return Ok(())
      } else {
        return Err("Player não encontrado");
      }
    }
    Err("O jogo não começou")
  }

}