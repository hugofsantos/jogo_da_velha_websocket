use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{filters::ws::{Message, WebSocket}, reply::Reply};

use crate::{Client, Clients, Result};

#[derive(Clone)]
pub struct ClientWebSocket {
  clients: Clients
}

impl ClientWebSocket {
  pub fn new(clients: Clients) -> Self {
    ClientWebSocket {clients}
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
      return;
    }
  }   
}