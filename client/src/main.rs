use std::{
    sync::mpsc::channel,
    thread,
};
use clap::{
    App,
    Arg,
};
use reqwest::blocking::Client as HttpClient;
use serde_json;
use websocket::{
    Message,
    sync::client::ClientBuilder as WsClientBuilder,
};
use ggez::{
    ContextBuilder,
    conf,
    event,
};
use ggezmulti::{
    GAME_AUTHOR,
    GAME_NAME,
    GAME_SERVER_HOST,
    GAME_SERVER_PORT,
    messages::{
        ClientMessage,
        RegisterRequest,
        RegisterResponse,
        ServerMessage,
    },
};

mod client;
mod input;
//mod resources;
//mod world;
//mod scenes;

use client::GameClient;

pub const DESIRED_FPS: u32 = 60;
pub const WINDOW_WIDTH: f32 = 800.;
pub const WINDOW_HEIGHT: f32 = 600.;

pub type Result<T> = std::result::Result<T, Box::<dyn std::error::Error>>;

fn main() {

    let matches = App::new("game-client")
        .help("game-client \"player name\"")
        .arg(Arg::with_name("player-name")
            .index(1)
            .required(true)
            .takes_value(true))
        .get_matches();

    let player_name = matches.value_of("player-name").unwrap();

    let register_response: RegisterResponse = HttpClient::new().post(
        &format!("http://{}:{}/register", GAME_SERVER_HOST, GAME_SERVER_PORT))
        .json(&RegisterRequest {
            player_name: player_name.into(),
        })
        .send().unwrap().json().unwrap();

    let game_con = WsClientBuilder::new(&register_response.url).unwrap().connect_insecure().unwrap();

    let (mut receiver, mut sender) = game_con.split().unwrap();

    let (server_tx, server_rx) = channel::<ServerMessage>();
    let (client_tx, client_rx) = channel::<ClientMessage>();

    let (mut ctx, event_loop) = ContextBuilder::new(GAME_NAME, GAME_AUTHOR)
        .window_setup(conf::WindowSetup::default().title(GAME_NAME))
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()
        .expect("aiieee");

    let event_handler = GameClient::new(&mut ctx, player_name.into(), server_rx, client_tx);

// receive incoming messages from server
    thread::spawn(move || {
        for msg in receiver.incoming_messages() {
            if let Ok(msg) = msg {
                let mut msg = Message::from(msg.clone());
                let string_payload = std::str::from_utf8(msg.payload.to_mut()).unwrap();
                let server_msg: ServerMessage = serde_json::from_str(&string_payload).unwrap();
                server_tx.send(server_msg).unwrap();
            }
        }
    });

// send outgoing messages from client
    thread::spawn(move || {
        loop {
            let client_msg = match client_rx.recv() {
                Ok(msg) => msg,
                Err(_) => break,
            };
            let _r = sender.send_message(&Message::text(serde_json::to_string(&client_msg).unwrap()));
        }
    });

    event::run(ctx, event_loop, event_handler);
}
