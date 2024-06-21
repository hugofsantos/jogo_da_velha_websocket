use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{filters::ws::{Message, WebSocket}, reply::Reply};

use crate::{Client, Clients, Games, Result};

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

    match client {
      Some(c) => Ok(
        ws.on_upgrade(
          move |socket| ClientWebSocket::client_connection(socket, id, clients, c)
        )
      ),
      None => Err(warp::reject::not_found())
    }
  }  

  pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
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

      ClientWebSocket::client_msg(&id, msg, &clients).await;      
    }

    clients.lock().await.remove(&id);
    println!("{id} desconectado");
  }  

  async fn client_msg(id: &str, msg: Message, clients: &Clients) {
    println!("Mensagem recebida de {}: {:?}", id, msg);

    let message = match msg.to_str() {
      Ok(v) => v,
      Err(_) => return
    };

    if message == "ping" || message == "ping\n" {
      ClientWebSocket::publish_msg_to_client(id, "pong", clients).await;
      return;
    }

    ClientWebSocket::publish_msg_by_game_id(message, message, clients).await;    
  }  

  async fn publish_msg_by_game_id(game_id: &str, msg: &str, clients: &Clients) {
    clients
      .lock()
      .await
      .iter_mut()
      .filter(|(_, client)| match &client.game_id {
        Some(game) => game == game_id,
        None => false
      })
      .for_each(|(_, client)| {
        if let Some(sender) = &client.sender {
          let _ = sender.send(Ok(Message::text(msg)));
        }
      });
  }

  async fn publish_msg_to_client(client_id: &str,msg: &str, clients: &Clients) {
    if let Some(c) = clients.lock().await.get(client_id).cloned() {
      if let Some(sender) = &c.sender {
          let _ = sender.send(Ok(Message::text(msg)));
      }
    }
  }    
}