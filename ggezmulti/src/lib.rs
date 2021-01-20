#![allow(dead_code)]
use serde::{
    Serialize, 
    Deserialize,
};

pub mod messages;

pub const GAME_NAME: &'static str = "YourGame";
pub const GAME_AUTHOR: &'static str = "You";

pub const GAME_SERVER_HOST: &'static str = "127.0.0.1";
pub const GAME_SERVER_PORT: u32 = 8191;

pub type PlayerName = String;
pub type GameId = String;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlayerState {
    // .
    // .
    // .
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState {
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    MyAction,
}
