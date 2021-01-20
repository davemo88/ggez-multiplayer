use std::{
    sync::Arc,
    time::Duration,
};
use reqwest;
use tokio::{
    sync::Mutex,
    time,
};
use std::collections::HashMap;
use ggezmulti::{
    GameId,
    PlayerName,
    PlayerState,
    messages::ServerMessage,
};
use crate::PUBLISH_URL;

type GameResult = Result<(), String>;

pub type Games = Arc<Mutex<HashMap<String, GameState>>>;

#[derive(Clone, Debug)]
pub struct GameState {
    pub player_states: HashMap<PlayerName, PlayerState>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            player_states: HashMap::new(),
        }
    }
}

pub async fn get_gamestate(game_id: &str, games: Games) -> Option<GameState> {
    games.lock().await.get(game_id).cloned()
}

pub async fn set_gamestate(game_id: String, state: GameState, games: Games) {
    games.lock().await.insert(game_id, state);
}

pub async fn gametask(game_id: GameId, games: Games) -> GameResult {
    let http_client = reqwest::Client::new();
    let game_over = false;
    while !game_over {
        let mut gs = get_gamestate(&game_id, games.clone()).await.unwrap();//.unwrap_or(return Err("unknown game".to_string()));
        // depends on your game
        // .
        // .
        // .
        set_gamestate(game_id.clone(), gs, games.clone()).await;
        let _r = http_client.post(PUBLISH_URL)
            .json(&ServerMessage {
                game_id: game_id.clone(),
                player_name: None,
                player_state:None,
            })
            .send().await.unwrap();
        time::delay_for(Duration::from_millis(100)).await;
    };
    Ok(())
}
