use futures::Future;
use uuid::Uuid;
use warp::reply::Reply;
use warp::http::StatusCode;

use crate::{Client, Clients, Result};

#[derive(Clone)]
pub struct ClientHandler {
  clients: Clients
}

impl ClientHandler {
  pub fn new(clients: Clients) -> Self {
    ClientHandler {clients}
  }

  pub fn health_handler(&self) -> impl Future<Output = Result<impl Reply>> {
    futures::future::ready(Ok(StatusCode::OK))
  }

  pub async fn register_client_handler(&self) -> Result<impl Reply> {
    let uuid = Uuid::new_v4().simple().to_string();

    self.register_client(uuid.clone()).await;
    Ok(uuid)
  }

  async fn register_client(&self, id: String) {
    self.clients.lock().await.insert(
      id,
      Client {
        topic: Some(String::from("teste\n")),
        sender: None
      }
    );
  }

  pub async fn unregister_client_handler(&self, id: String) -> Result<impl Reply> {
    let removed_client = self.clients.lock().await.remove(&id);

    match removed_client {
      Some(_) => Ok(StatusCode::OK),
      None => Err(warp::reject::not_found())
    }
  }
}