use futures::Future;
use uuid::Uuid;
use warp::reply::Reply;
use warp::http::StatusCode;

use crate::{ws, Client, Clients, Result};

pub fn health_handler() -> impl Future<Output = Result<impl Reply>> {
  futures::future::ready(Ok(StatusCode::OK))
}

pub async fn register_client_handler(clients: Clients) -> Result<impl Reply> {
  let uuid = Uuid::new_v4().simple().to_string();

  register_client(uuid.clone(), clients).await;
  Ok(uuid)
}

async fn register_client(id: String, clients: Clients) {
  clients.lock().await.insert(
    id,
    Client {
      sender: None
    }
  );
}

pub async fn unregister_client_handler(id: String, clients:Clients) -> Result<impl Reply> {
  clients.lock().await.remove(&id);
  Ok(StatusCode::OK)
}

// Websocket

pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply> {
  let client = clients.lock().await.get(&id).cloned();

  match client {
    Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c))),
    None => Err(warp::reject::not_found())
  }
}