use futures::{
    FutureExt,
    StreamExt,
};
use serde_json::{
    from_str,
    self,
};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::{
    http::StatusCode,
    reply::json,
    ws::{Message, Ws, WebSocket},
    Reply,
};
use ggezmulti::{
    GAME_SERVER_PORT,
    PlayerState,
    messages::{
        ClientMessage,
        RegisterRequest,
        RegisterResponse,
        ServerMessage,
    },
};
use crate::{
    gamestate::{
        gametask,
        get_gamestate,
        set_gamestate,
        Games,
        GameState,
    },
    actions::player_action,
    Client,
    Clients,
    Result,
};

pub async fn register_handler(body: RegisterRequest, clients: Clients, games: Games) -> Result<impl Reply> {

    match get_gamestate(&body.game_id, games.clone()).await {
        Some(mut gs) => {
// join existing game
            gs.player_states.insert(body.player_name.clone(), PlayerState::default());
            set_gamestate(body.game_id.clone(), gs, games).await;
        },
        None => {
// create new game
            set_gamestate(body.game_id.clone(), GameState::new(), games.clone()).await;
            tokio::task::spawn(gametask(body.game_id.clone(), games));
        },
    }
    
    let token = Uuid::new_v4().simple().to_string();
    register_client(token.clone(), body.player_name, body.game_id, clients).await;

    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:{}/ws/{}", GAME_SERVER_PORT, token),
    }))
}

async fn register_client(token: String, player_name: String, game_id: String, clients: Clients) {
    clients.lock().await.insert(
        token,
        Client {
            player_name,
            game_id,
            sender: None,
        },
    );
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply> {
    clients.lock().await.remove(&id);
    Ok(StatusCode::OK)
}

pub async fn ws_handler(ws: Ws, id: String, clients: Clients, games: Games) -> Result<impl Reply> {
    let client = clients.lock().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c, games))),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client, games: Games) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender.clone());
    clients.lock().await.insert(id.clone(), client.clone());

    println!("{} connected", id);

    let mut gs = get_gamestate(&client.game_id, games.clone()).await.unwrap();
    let mut ps = PlayerState::default();
    gs.player_states.insert(client.player_name.clone(), ps.clone());
    set_gamestate(client.game_id.clone(), gs, games.clone()).await;
    let _r = client_sender.send(Ok(Message::text(serde_json::to_string(&ServerMessage {
        player_name: Some(client.player_name),
        game_id: client.game_id,
        player_state: Some(ps),
    }).unwrap())));

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
              eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
              break;
              }
        };
        client_msg(&id, msg, &clients, &games).await;
    }

    clients.lock().await.remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients, games: &Games) {
    let message = match msg.to_str() {
      Ok(v) => v,
      Err(_) => return,
    };

    if message == "ping" || message == "ping\n" {
      return;
    }

    match from_str::<ClientMessage>(&message) {
        Ok(msg) => { 
            println!("received message from {}: {:?}", id, msg);
            let gs = player_action(msg.player_name.clone(), msg.game_id.clone(), msg.action, games.clone()).await;
            let _r = clients.lock().await.get(id).unwrap().sender.as_ref().unwrap().send(Ok(Message::text(serde_json::to_string(&ServerMessage {
                player_name: Some(msg.player_name.clone()),
                game_id: msg.game_id.clone(),
                player_state: Some(gs.player_states.get(&msg.player_name).unwrap().clone()), 
            }).unwrap())));
        }
        Err(e) => {
          eprintln!("error while parsing message to input request: {}", e);
          return;
        }
    };
}

pub async fn publish_handler(server_msg: ServerMessage, clients: Clients) -> Result<impl Reply> {
    clients
        .lock()
        .await
        .iter_mut()
        .filter(|(_, client)| client.game_id == server_msg.game_id)
        .filter(|(_, client)| if let Some(name) = server_msg.player_name.clone() { client.player_name == name } else { true })
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
              let _ = sender.send(Ok(Message::text(serde_json::to_string(&server_msg).unwrap())));
            }
        });
    Ok(StatusCode::OK)
}
