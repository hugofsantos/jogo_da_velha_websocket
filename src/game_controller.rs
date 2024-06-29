use std::{collections::HashMap, result};

use uuid::Uuid;

use crate::{game::{Game, ResultOfTheMove}, Clients, Games};

pub enum ResultOfAddPlayerToGame {
  PlayerAdded {player_symbol: char, number_of_players: usize},
  Error (&'static str)
}

pub async fn join_game(player_id: &str, clients: &Clients, games: &Games) -> ResultOfAddPlayerToGame {
  let mut games_guard = games.lock().await;
  let game_with_single_player = games_guard.iter_mut()
    .find(|(_,game)| game.number_of_players() == 1);

  if let Some((game_id, game)) = game_with_single_player {
    add_player_to_game(player_id, game_id, game, clients).await
  } else {
    match create_game_with_one_player(player_id, clients, &mut games_guard).await {
      Ok(_) => ResultOfAddPlayerToGame::PlayerAdded { player_symbol: 'X', number_of_players: 1 },
      Err(msg) => ResultOfAddPlayerToGame::Error(msg)
    }
  }
}

async fn add_player_to_game(player_id: &str, game_id:&str, game: &mut Game, clients: &Clients) -> ResultOfAddPlayerToGame {
  let mut client_guard = clients.lock().await;
  let client = client_guard.get_mut(player_id);

  if let Some(c) = client {
    if let None = c.game_id {
      match game.add_player(String::from(player_id)) {
        Ok(symbol) => {
          c.game_id = Some(String::from(game_id));
          ResultOfAddPlayerToGame::PlayerAdded { 
            player_symbol: symbol, 
            number_of_players: game.number_of_players() 
          }
        },
        Err(msg) => ResultOfAddPlayerToGame::Error(msg)
      }
    } else {
      ResultOfAddPlayerToGame::Error("Esse jogador já está em uma partida")
    }
  } else {
    ResultOfAddPlayerToGame::Error("Não foi encontrado nenhum jogador com esse ID")
  }
}

pub async fn remove_player_from_game(player_id: &str, clients: &Clients, games: &Games) -> Result<(), &'static str> {
  let mut client_guard = clients.lock().await;
  let client = client_guard.get_mut(player_id);

  if let Some(c) = client {
    if let Some(game_id) = &c.game_id {
      let mut games_guard = games.lock().await;
      let game = games_guard.get_mut(game_id);

      if let Some(g) = game {
        g.remove_player(player_id)?;

        if g.number_of_players() == 0 {
          games_guard.remove(game_id);
        }
      }
    }    

    c.game_id = None;
    Ok(())
  } else {
    Err("Não foi encontrado nenhum jogador com esse ID")
  }
}

async fn create_game_with_one_player(player_id: &str, clients: &Clients, games: &mut HashMap<String, Game>) -> Result<(), &'static str> {
    let mut client_guard = clients.lock().await;
    let client = client_guard.get_mut(player_id);

    if let Some(c) = client {
      if let None = c.game_id {
        let uuid = Uuid::new_v4().simple().to_string();
        let game = Game::new();

        games.insert(uuid, game);
        Ok(())
      } else {
        Err("Esse jogador já está em uma partida")
      }
    } else {
      Err("Não foi encontrado nenhum jogador com esse ID")
    }    
}

pub async fn make_move(player_id: &str, clients: &Clients, games: &Games, position: usize) -> Result<ResultOfTheMove,  &'static str> {
  let mut client_guard = clients.lock().await;
  let client = client_guard.get_mut(player_id);
  let mut games_guard = games.lock().await;

  if let Some(c) = client {
    let game_id = match c.game_id.clone() {
      Some(s) => s,
      None => return Err("ID do jogo não rechonecido"),
    };
    let game_id = game_id.as_str();

    let game = match games_guard.get_mut(game_id){
      Some(g) => g,
      None => return Err("Jogo não encontrado")
    };
    let row = position / 3;
    let col = position % 3;
    let result = game.make_move(player_id, row, col);
    Ok(result)

  } else {
    Err("Cliente não encontrado")
  } 
}
