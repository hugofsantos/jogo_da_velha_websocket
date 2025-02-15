use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{filters::ws::{Message, WebSocket}, reply::Reply};

use crate::{game::ResultOfTheMove, game_controller::{join_game, make_move, remove_player_from_game, ResultOfAddPlayerToGame}, Client, Clients, Command, Games, ParseCommandError, Result};

#[derive(Clone)]
pub struct ClientWebSocket {
  clients: Clients,
  games: Games
}

impl ClientWebSocket {
  pub fn new(clients: Clients, games: Games) -> Self {
    ClientWebSocket {clients, games}
  }  

  pub async fn ws_handler(&self, ws: warp::ws::Ws, id: String) -> Result<impl Reply> {
    let clients = self.clients.clone();
    let client = clients.lock().await.get(&id).cloned();
    let games = self.games.clone();

    match client {
      Some(c) => Ok(
        ws.on_upgrade(
          move |socket| ClientWebSocket::client_connection(socket, id, clients, c, games)
        )
      ),
      None => Err(warp::reject::not_found())
    }
  }  

  pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client, games: Games) {
    let (client_ws_sender, mut client_ws_receiver) = ws.split();
    let (client_sender, client_receiver) = mpsc::unbounded_channel();

    let client_receiver = UnboundedReceiverStream::new(client_receiver);

    tokio::task::spawn(client_receiver.forward(client_ws_sender).map(|result| {
      if let Err(e) = result {
        eprintln!("Erro ao enviar mensagem no websocket: {}", e);
      }
    }));
    
    client.sender = Some(client_sender);
    clients.lock().await.insert(id.clone(), client);
    println!("{id} connected!");

    while let Some(result) = client_ws_receiver.next().await {
      let msg = match result {
        Ok(message) => message,
        Err(e) => {
          eprintln!("Erro ao receber mensagem no websocket pelo id: {}: {}", id.clone(), e);
          break;
        }
      };


      ClientWebSocket::client_msg(&id, msg, &clients, &games).await;      
    }

    ClientWebSocket::disconnect_client(&id, &clients, &games).await;
  }  

  async fn client_msg(id: &str, msg: Message, clients: &Clients, games: &Games) {
    println!("Mensagem recebida de {}: {:?}", id, msg);

    let message = match msg.to_str() {
      Ok(v) => v,
      Err(_) => return
    };

    match Command::from_str(message) {
      Ok(Command::MakeMove {position }) =>  {
        let result = make_move(id, clients, games, position).await;       
        let game_id = ClientWebSocket::get_game_id_by_client_id(id, clients)
          .await;

       match result {
          Ok(ResultOfTheMove::Draw) => {
            let message = "draw";

            ClientWebSocket::publish_msg_by_game_id(game_id.as_str(), message, clients).await;
          },
          Ok(ResultOfTheMove::MarkedCell(symbol)) => {
            let message = format!("marked {symbol} {position}");
            let message = message.as_str();

            ClientWebSocket::publish_msg_by_game_id(game_id.as_str(), message, clients).await;
          },
          Ok(ResultOfTheMove::Win(symbol)) => {
            let message = format!("winner {symbol}");
            let message = message.as_str();

            ClientWebSocket::publish_msg_by_game_id(game_id.as_str(), message, clients).await;
          },
          Ok(ResultOfTheMove::Error(message)) => {
            ClientWebSocket::publish_msg_by_game_id(game_id.as_str(), message, clients).await;
          },
          Err(message) => {
            ClientWebSocket::publish_msg_by_game_id(game_id.as_str(), message, clients).await;
          }
       }
      },
      Ok(Command::JoinGame) => {
        let result = join_game(id, clients, games).await;

        match result {
            ResultOfAddPlayerToGame::PlayerAdded { player_symbol, number_of_players, game_id } => {
              let message = format!("{player_symbol}");
              let message = message.as_str();

              ClientWebSocket::publish_msg_to_client(id, message, clients).await;

              let message = if number_of_players == 1 {"waiting_players"} else {"start_game"};

              ClientWebSocket::publish_msg_by_game_id(game_id.as_str(), message, clients).await;
            },
            ResultOfAddPlayerToGame::Error(message) => {
              ClientWebSocket::publish_msg_to_client(id, message, clients).await;
            },
        }


      },
      Err(ParseCommandError::InvalidCommand) => {
        let message = format!("Comando {message} inválido");
        let message = message.as_str();
        
        ClientWebSocket::publish_msg_to_client(id, message, clients).await;
      },
      Err(ParseCommandError::InvalidParameters) => {
        let message = format!("Parâmetros para o comando '{message}' são inválidos");
        let message = message.as_str();

        ClientWebSocket::publish_msg_to_client(id, message, clients).await;
      },
    }  
  }  

  async fn publish_msg_by_game_id(game_id: &str, msg: &str, clients: &Clients) {
    clients
      .lock()
      .await
      .iter_mut()
      .filter(|(_, client)| match &client.game_id {
        Some(game) => game.as_str() == game_id,
        None => false
      })
      .for_each(|(_, client)| {
        if let Some(sender) = &client.sender {
          let _ = sender.send(Ok(Message::text(msg)));
          println!("MENSAGEM ENVIADA NO TÓPICO {game_id}: {msg}");
        }
      });
  }

  async fn publish_msg_to_client(client_id: &str,msg: &str, clients: &Clients) {
    println!("MENSAGEM ENVIADA: {msg}");
    if let Some(c) = clients.lock().await.get(client_id).cloned() {
      if let Some(sender) = &c.sender {
          let _ = sender.send(Ok(Message::text(msg)));
      }
    }
  }    

  async fn get_game_id_by_client_id(client_id: &str, clients: &Clients) -> String {
      match clients.lock().await.get(client_id) {
          Some(client) => client.game_id.clone().unwrap_or(String::new()),
          None => String::new()
      }
  }  

  async fn disconnect_client(client_id: &str, clients: &Clients, games: &Games) {
    let game_id = ClientWebSocket::get_game_id_by_client_id(client_id, clients).await;
    let _ = remove_player_from_game(client_id, &clients, &games).await;
    ClientWebSocket::publish_msg_by_game_id(game_id.as_str(), "X", clients).await;
    ClientWebSocket::publish_msg_by_game_id(game_id.as_str(), "waiting_players", clients).await;
    clients.lock().await.remove(client_id);
    println!("{client_id} desconectado");    
  }
}
