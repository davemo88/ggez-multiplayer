use std::{
    collections::HashMap,
    convert::Infallible,
    sync::Arc,
};
use tokio::sync::{
    Mutex,
    mpsc,
};
use warp::{
    ws::Message,
    Filter,
    Rejection,
};
use ggezmulti::{
    GameId,
    PlayerName,
};
mod actions;
mod gamestate;
mod handlers;
use crate::{
    gamestate::Games,
    handlers::{
        publish_handler,
        register_handler,
        unregister_handler,
        ws_handler,
    },
};

pub const PUBLISH_URL: &'static str = "http://127.0.0.1:8191/publish";

pub type Result<T> = std::result::Result<T, Rejection>;

#[derive(Clone, Debug)]
pub struct Client {
   pub player_name: PlayerName,
   pub game_id: GameId,
   pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub type Clients = Arc<Mutex<HashMap<PlayerName, Client>>>;

fn with<T>(shared_resource: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone where T: Clone + Send {
    warp::any().map(move || shared_resource.clone())
}

#[tokio::main]
async fn main() {

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let games: Games = Arc::new(Mutex::new(HashMap::new()));

    let register = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with(clients.clone()))
        .and(with(games.clone()))
        .and_then(register_handler);

    let unregister = warp::path("unregister")
        .and(warp::post())
        .and(warp::body::json())
        .and(with(clients.clone()))
        .and_then(unregister_handler);

    let ws = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with(clients.clone()))
        .and(with(games.clone()))
        .and_then(ws_handler);

    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with(clients.clone()))
        .and_then(publish_handler);

    let routes = register
        .or(unregister)
        .or(ws)
        .or(publish)
        .with(warp::cors().allow_any_origin());
    
    warp::serve(routes).run(([127,0,0,1], 8191)).await
}
