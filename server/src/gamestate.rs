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
use crate::{
    PUBLISH_URL,
    Games,
};

type GameResult = Result<(), String>;

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

pub async fn gametask(game_id: GameId, games: Games) -> GameResult {
    let http_client = reqwest::Client::new();
    let game_over = false;
    while !game_over {
        let mut games = games.lock().await;
        let gs = games.get_mut(&game_id).unwrap();
        // .
        // .
        // .
        let _r = http_client.post(PUBLISH_URL)
            .json(&ServerMessage::PressedA)
            .send().await.unwrap();
        time::sleep(Duration::from_millis(100)).await;
    };
    Ok(())
}
