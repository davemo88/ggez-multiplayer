use ggezmulti::{
    Action,
    GameId,
    PlayerName,
};
use crate::{
    Games,
    gamestate::{
        get_gamestate,
        set_gamestate,
        GameState,
    },
};

pub async fn player_action(_player_name: PlayerName, game_id: GameId, action: Action, games: Games) -> GameState {
    let mut gs = get_gamestate(&game_id, games.clone()).await.unwrap();
    match action {
        Action::MyAction => (),
    };
    set_gamestate(game_id, gs.clone(), games).await;
    gs
}

