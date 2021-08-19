use std::{
    collections::{HashMap, VecDeque},
    convert::Infallible,
    sync::Arc,
    time::Duration,
};
use serde::{Serialize, Deserialize};
use simple_logger::SimpleLogger;
use log::{LevelFilter, debug};
use tokio::sync::{
    Mutex,
    RwLock,
    mpsc,
};
use warp::{
    ws::Message,
    Filter,
    Rejection,
};
use uuid::Uuid;
use ggezmulti::{
    GameId,
    PlayerName,
    messages::ServerMessage,
};
mod error;
mod gamestate;
mod handlers;
use crate::{
    handlers::{
        publish_handler,
        register_handler,
        unregister_handler,
        ws_handler,
    },
    gamestate::{
        GameState,
        gametask,
    },
};

pub const PUBLISH_URL: &'static str = "http://127.0.0.1:8191/publish";

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type WebResult<T> = std::result::Result<T, Rejection>;

#[derive(Clone, Debug)]
pub struct Client {
   pub player_name: PlayerName,
   pub game_id: Option<GameId>,
   pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub type Clients = Arc<Mutex<HashMap<PlayerName, Client>>>;
pub type Matchmaking = Arc<Mutex<VecDeque<String>>>;
pub type Games = Arc<Mutex<HashMap<String, GameState>>>;

fn with<T>(shared_resource: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone where T: Clone + Send {
    warp::any().map(move || shared_resource.clone())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishParams {
    player_name: Option<PlayerName>,
    game_id: Option<GameId>,
}

pub async fn publish_to_player(client: &reqwest::Client, player: &PlayerName, msg: ServerMessage) -> reqwest::Result<reqwest::Response> {
    client.post(&format!("{}/?player_name={}", PUBLISH_URL, player))
        .json(&msg)
        .send().await
}

pub async fn publish_to_game(client: &reqwest::Client, game_id: &GameId, msg: ServerMessage) -> reqwest::Result<reqwest::Response> {
    client.post(&format!("{}/?game_id={}", PUBLISH_URL, game_id))
        .json(&msg)
        .send().await
}

#[tokio::main]
async fn main() {

    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_module_level("hyper", LevelFilter::Warn)
        .with_module_level("reqwest", LevelFilter::Warn)
        .with_module_level("warp", LevelFilter::Warn)
        .init()
        .unwrap();

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let games: Games = Arc::new(Mutex::new(HashMap::new()));
    let matchmaking: Matchmaking = Arc::new(Mutex::new(VecDeque::new()));

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
        .and(with(matchmaking.clone()))
        .and_then(ws_handler);

    let publish = warp::path("publish")
        .and(warp::body::json())
        .and(with(clients.clone()))
        .and(warp::query::<PublishParams>())
        .and_then(publish_handler);

    let routes = register
        .or(unregister)
        .or(ws)
        .with(warp::cors().allow_any_origin())
// does this do what i want?
        .or(publish);

// spawn matchmaking task
    tokio::spawn(async move {
        let http_client = reqwest::Client::new();
        loop {
            let mut mm = matchmaking.lock().await;
            if mm.len() > 1 {
// TODO: need persistent user UUID's or either the players must not have the same name
// or they won't be able to disconnect / reconnect
                let (p1_id, p2_id) = (mm.pop_front().unwrap(), mm.pop_front().unwrap());
                drop(mm);
                let mut clients = clients.lock().await;
                let game_id = Uuid::new_v4().simple().to_string();
                let p1 = {
                    let p1_client = clients.get_mut(&p1_id).unwrap();
                    p1_client.game_id = Some(game_id.clone());
                    p1_client.player_name.clone()
                };
                let p2 = {
                    let p2_client = clients.get_mut(&p2_id).unwrap();
                    p2_client.game_id = Some(game_id.clone());
                    p2_client.player_name.clone()
                };
                drop(clients);
                debug!("starting match between {} and {}", p1, p2);
                let gs = GameState::new([p1.clone(), p2.clone()]);
                games.lock().await.insert(game_id.clone(), gs);
                tokio::task::spawn(gametask(game_id.clone(), games.clone()));
                publish_to_player(&http_client, &p1, ServerMessage::MatchFound(p2.clone())).await.unwrap();
                publish_to_player(&http_client, &p2, ServerMessage::MatchFound(p1.clone())).await.unwrap();
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    
    warp::serve(routes).run(([127,0,0,1], 8191)).await
}
