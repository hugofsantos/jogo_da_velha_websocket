use crate::{Clients, Games};

pub async fn add_player_to_a_game(player_id: &str, clients: &Clients, games: &Games) -> Result<char, &'static str> {
  let mut client_guard = clients.lock().await;
  let client = client_guard.get_mut(player_id);

  if let Some(c) = client {
    if let Some(_) = c.game_id  {
      let mut games_guard = games.lock().await;
      let game_with_single_player = games_guard.iter_mut()
        .find(|(_, game)| game.number_of_players() == 1);

      if let Some((game_id, game)) = game_with_single_player {
        return match game.add_player(String::from(player_id)) {
          Ok(player_symbol) => {
            c.game_id = Some(game_id.clone());
            Ok(player_symbol)
          },
          Err(msg) => Err(msg)
        }
      } else {
        // TODO: Criar uma partida e colocar o player nela
      }
    } else {
      return Err("Este jogador já está em uma partida");
    }
  } else {
    return Err("Cliente não encontrado");
  }
  Ok('X')
}