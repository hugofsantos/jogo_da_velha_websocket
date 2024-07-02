use std::collections::HashMap;

pub enum ResultOfTheMove {
  MarkedCell(char),
  Win(char),
  Draw,
  Error(&'static str)
}

#[derive(Clone)]
pub struct Game {
  board: [[char; 3];3],
  current_turn: char,
  players: HashMap<String, char>
}

impl Game {
  pub fn new() -> Self {
    Game {
      board: [[' '; 3]; 3],
      current_turn: 'X',
      players: HashMap::new()
    }
  }

  pub fn add_player(&mut self, player_id: String) -> Result<char, &'static str> {
    if self.players.len() >= 2 {
      return Err("O jogo está completo");
    }

    let symbol = if self.players.len() == 0 {'X'} else {'O'};
    self.players.insert(player_id, symbol);
    Ok(symbol)
  }

  pub fn remove_player(&mut self, player_id: &str) -> Result<(), &'static str> {
    if self.players.remove(player_id).is_some() {
      if self.players.len() == 1 {
        let remaining_player_id = self.players.keys().next().unwrap();

        self.players.insert(remaining_player_id.clone(), 'X');
        self.reset_game();
      }   

      Ok(())
    } else {
      Err("Player não encontrado")
    }
  }

  fn reset_game(&mut self) {
    self.board = [[' '; 3]; 3];
    self.current_turn = 'X';
  }

  pub fn number_of_players(&self) -> usize {
    self.players.len()
  }

  pub fn make_move(&mut self, player_id: &str, row: usize, col: usize) -> ResultOfTheMove{
    if self.board[row][col] != ' ' {
      return ResultOfTheMove::Error("Esta célula já foi preenchida")
    }

    if let Some(&symbol) = self.players.get(player_id) {
      if symbol != self.current_turn {
        return ResultOfTheMove::Error("Não é sua vez de jogar")
      }

      self.board[row][col] = symbol; 
      self.current_turn = if self.current_turn == 'X' {'O'} else {'X'};

      if self.is_winner(player_id) {
        return {self.reset_game(); ResultOfTheMove::Win(symbol)}
      }

      if self.board_is_filled() {
        return {self.reset_game() ;ResultOfTheMove::Draw}
      }

      ResultOfTheMove::MarkedCell(symbol)
    } else {
      return ResultOfTheMove::Error("Não existe nenhum player com esse ID no jogo");
    } 
  }

  fn board_is_filled(&self) -> bool {
    let board_size = self.board.len();

    for l in 0..board_size {
      for c in 0..board_size {
        if self.board[l][c] == ' ' { // Se não está preenchido
          return false;
        }
      }
    }
    
    true
  }

  fn is_winner(&self, player_id: &str) -> bool {
    if let Some(player_symbol) = self.players.get(player_id) {
      self.win_by_horizontal(*player_symbol) ||
      self.win_by_vertical(*player_symbol) ||
      self.win_by_principal_diagonal(*player_symbol) ||
      self.win_by_secondary_diagonal(*player_symbol)
    } else {
      false
    }
  }

  fn win_by_horizontal(&self, player_symbol: char) -> bool {
    let board_size = self.board.len();
    
    for l in 0..board_size {
      let mut win = true;

      for c in 0..board_size {
        if self.board[l][c] != player_symbol {
          win = false;
          break;
        }
      }

      if win {
        return true;
      }
    }    
    
    false
  }

  fn win_by_vertical(&self, player_symbol: char) -> bool {
    let board_size = self.board.len();

    for c in 0..board_size {
      let mut win = true;

      for l in 0..board_size {
        if self.board[l][c] != player_symbol {
          win = false;
          break;
        }
      }

      if win {
        return true;
      }
    }
    
    false  
  }

  fn win_by_principal_diagonal(&self, player_symbol: char) -> bool {
    for i in 0..self.board.len() {
      if self.board[i][i] != player_symbol {
        return false;
      }
    }

    true
  }

  fn win_by_secondary_diagonal(&self, player_symbol: char) -> bool {
    let board_size = self.board.len();
    
    for l in 0..board_size {
      let c = (board_size - 1) - l;

      if self.board[l][c] != player_symbol {
        return false;
      }
    }

    true
  }

}