use serde::{
    Serialize,
    Deserialize,
};
use crate::PlayerName;


#[derive(Serialize, Deserialize)]
pub struct RegisterRequest {
  pub player_name: PlayerName,
}

#[derive(Serialize, Deserialize)]
pub struct RegisterResponse {
  pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    PressedA,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    PressedA,
}
