use std::sync::mpsc::{
    Receiver,
    Sender,
};
use ggez::{
    Context,
    GameResult,
    event::{KeyCode, EventHandler},
    input::keyboard::KeyMods,
    graphics,
    timer,
};
use ggezmulti::{
    Action,
    GameId,
    PlayerName,
    PlayerState,
    messages::{
        ClientMessage,
        ServerMessage,
    },
};

use crate::{
    DESIRED_FPS,
    input,
};

pub struct GameClient {
    player_name: PlayerName,
    game_id: GameId,
    server_msg_rx: Receiver<ServerMessage>,
    client_msg_tx: Sender<ClientMessage>,
    player_state: PlayerState,
    input_state: input::State, 
}

impl GameClient {
    pub fn new(_ctx: &mut Context, player_name: PlayerName, game_id: GameId, server_msg_rx: Receiver<ServerMessage>, client_msg_tx: Sender<ClientMessage>) -> Self { 

        GameClient {
            player_name,
            game_id,
            server_msg_rx,
            client_msg_tx,
            player_state: PlayerState::default(),
            input_state: input::create_input_state(), 
        }
    }

    pub fn handle_server_msg(&mut self, msg: ServerMessage) {
        assert_eq!(msg.game_id, self.game_id);
        if let Some(player_name) = msg.player_name {
            assert_eq!(player_name, self.player_name);
            self.player_state = msg.player_state.unwrap();
        }
    }

}

impl EventHandler for GameClient {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if timer::check_update_time(ctx, DESIRED_FPS) {
            if let Ok(msg) = self.server_msg_rx.try_recv() {
                self.handle_server_msg(msg);
            }
        }
        if self.input_state.get_default_player_button_pressed(input::GameButton::A) {
            let _r = self.client_msg_tx.send(ClientMessage {
                player_name: self.player_name.clone(),
                game_id: self.game_id.clone(),
                action: Action::MyAction,
            });
        }
        self.input_state.update(timer::delta(ctx).as_secs_f32());
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        // .
        // .
        // .
        graphics::present(ctx)
    }

    fn key_down_event(&mut self, _ctx: &mut Context, key: KeyCode, _keymod: KeyMods, _repeat: bool) {
        self.input_state.update_key_down(key);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, key: KeyCode, _keymod: KeyMods,) {
        self.input_state.update_key_up(key);
    }
}


