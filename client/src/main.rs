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
    GameId,
    PlayerName,
    messages::{
        ClientMessage,
        RegisterRequest,
        RegisterResponse,
        ServerMessage,
    },
};

mod client;
mod input;

use client::GameClient;

pub const DESIRED_FPS: u32 = 60;
pub const WINDOW_WIDTH: f32 = 800.;
pub const WINDOW_HEIGHT: f32 = 600.;

fn main() {

    let matches = App::new("game-client")
        .help("game-client \"player name\" \"game id\"")
        .arg(Arg::with_name("player-name")
            .index(1)
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("game-id")
            .index(2)
            .required(true)
            .takes_value(true))
        .get_matches();

    let (player_name, game_id): (PlayerName, GameId) = (matches.value_of("player-name").unwrap().into(), matches.value_of("game-id").unwrap().into());

    let register_response: RegisterResponse = HttpClient::new().post(
        &format!("http://{}:{}/register", GAME_SERVER_HOST, GAME_SERVER_PORT))
        .json(&RegisterRequest {
            player_name: player_name.clone(),
            game_id: game_id.clone(),
        })
        .send().unwrap().json().unwrap();

    let game_con = WsClientBuilder::new(&register_response.url).unwrap().connect_insecure().unwrap();

    let (mut receiver, mut sender) = game_con.split().unwrap();

    let (server_tx, server_rx) = channel::<ServerMessage>();
    let (client_tx, client_rx) = channel::<ClientMessage>();

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new(GAME_NAME, GAME_AUTHOR)
        .conf(conf::Conf::new())
        .window_setup(conf::WindowSetup::default().title(GAME_NAME))
        .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()
        .expect("aiieee");

    let event_handler = &mut GameClient::new(ctx, player_name.clone(), game_id.clone(), server_rx, client_tx);

// receive incoming messages from server
    thread::spawn(move || {
        for msg in receiver.incoming_messages() {
            if let Ok(msg) = msg {
                let mut msg = Message::from(msg.clone());
                let string_payload = std::str::from_utf8(msg.payload.to_mut()).unwrap();
                let server_msg: ServerMessage = serde_json::from_str(&string_payload).unwrap();
                println!("server_msg: {:?}", server_msg);
                let _r = server_tx.send(server_msg);
            }
        }
    });

// send outgoing messages from client
    thread::spawn(move || {
        loop {
            let client_msg = client_rx.recv().unwrap();
            let _r = sender.send_message(&Message::text(serde_json::to_string(&client_msg).unwrap()));
        }
    });

    match event::run(ctx, event_loop, event_handler) {
        Ok(_) => (),
        Err(e) => println!("Error occurred: {}", e)
    };
}

