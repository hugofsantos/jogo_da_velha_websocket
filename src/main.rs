use std::{collections::HashMap, convert::Infallible, sync::Arc};

use handler::ClientHandler;
use tokio::sync::{mpsc, Mutex};
use warp::{filters::ws::Message, reject::Rejection, Filter};
use ws::ClientWebSocket;

mod handler;
mod ws;

#[derive(Clone)]
pub struct Client {
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>
}

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<Mutex<HashMap<String, Client>>>;

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let client_handler = ClientHandler::new(clients.clone());
    let client_websocket = ClientWebSocket::new(clients.clone());

    let health_route = warp::path!("health")
    .and(with_handler(client_handler.clone()))
    .and_then(|handler: ClientHandler| async move {
        handler.health_handler().await
    });

    let register_client = warp::path("register");
    let register_client_routes = register_client
        .and(warp::post())
        .and(with_handler(client_handler.clone()))
        .and_then(|handler: ClientHandler| async move {
            handler.register_client_handler().await
        })
        .or(register_client
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_handler(client_handler.clone()))
            .and_then(|client_id: String, handler: ClientHandler| async move {
                handler.unregister_client_handler(client_id).await
            })
        );
    
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_websocket(client_websocket.clone()))
        .and_then(|ws: warp::ws::Ws, id: String, websocket: ClientWebSocket| async move {
            websocket.ws_handler(ws, id).await
        });

    let routes = health_route
        .or(register_client_routes)
        .or(ws_route);
    
    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_handler(handler: ClientHandler) -> impl Filter<Extract = (ClientHandler,), Error=Infallible> + Clone {
    warp::any().map(move || handler.clone())
}

fn with_websocket(ws: ClientWebSocket) -> impl Filter<Extract = (ClientWebSocket,), Error=Infallible> + Clone {
    warp::any().map(move || ws.clone())
}
