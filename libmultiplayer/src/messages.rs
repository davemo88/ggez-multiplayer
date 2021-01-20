use serde::{
    Serialize,
    Deserialize,
};
use crate::{
    Action,
    GameId,
    PlayerName,
    PlayerState,
};


#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
  pub player_name: PlayerName,
  pub game_id: GameId,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
  pub url: String,
}

pub type FightRecord = Vec::<String>;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientMessage {
    pub player_name: PlayerName,
    pub game_id: GameId,
    pub action: Action,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerMessage {
    pub game_id: GameId,
    pub player_name: Option<PlayerName>,
    pub player_state: Option<PlayerState>,
}
