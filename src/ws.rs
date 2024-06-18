use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::{self};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::filters::ws::{Message, WebSocket};

use crate::{Client, Clients};

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

    client_msg(&id, msg, &clients).await;      
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

  // TODO: Adicionar lógica de mensagens
} 

// TODO: Função para enviar dados pelo websocket