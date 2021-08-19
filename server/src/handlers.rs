use futures::{
    FutureExt,
    StreamExt,
};
use log::{debug, error};
use serde_json::{
    from_str,
    self,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    http::StatusCode,
    reply::json,
    ws::{Message, Ws, WebSocket},
    Reply,
};
use ggezmulti::{
    GAME_SERVER_PORT,
    messages::{
        ClientMessage,
        RegisterRequest,
        RegisterResponse,
        ServerMessage,
    },
};
use crate::{
    error::Error,
    gamestate::{
        gametask,
        GameState,
    },
    Client,
    Clients,
    Games,
    Matchmaking,
    Result,
    WebResult,
    PublishParams,
    publish_to_player,
};

pub async fn register_handler(body: RegisterRequest, clients: Clients, games: Games) -> WebResult<impl Reply> {
    let token = Uuid::new_v4().simple().to_string();
    register_client(token.clone(), body.player_name, clients).await;

    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:{}/ws/{}", GAME_SERVER_PORT, token),
    }))
}

async fn register_client(token: String, player_name: String, clients: Clients) {
    clients.lock().await.insert(
        token,
        Client {
            player_name,
            game_id: None,
            sender: None,
        },
    );
}

pub async fn unregister_handler(id: String, clients: Clients) -> WebResult<impl Reply> {
    clients.lock().await.remove(&id);
    Ok(StatusCode::OK)
}

pub async fn ws_handler(ws: Ws, id: String, clients: Clients, games: Games, matchmaking: Matchmaking) -> WebResult<impl Reply> {
    let client = clients.lock().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c, games, matchmaking))),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client, games: Games, matchmaking: Matchmaking) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender.clone());
    clients.lock().await.insert(id.clone(), client.clone());

    debug!("{} connected", id);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("error receiving ws message for id: {}): {}", &id, e);
                break;
              }
        };
        match client_msg(&id, msg, &clients, &games, &matchmaking).await {
            Ok(()) => (),
            Err(e) => error!("error handling client message: {:?}", e),
        };
    }
    matchmaking.lock().await.retain(|name| *name != client.player_name);
    clients.lock().await.remove(&id);
    debug!("{} disconnected", &id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients, games: &Games, matchmaking: &Matchmaking) -> Result<()> {
    let msg = msg.to_str().unwrap();
    let msg = from_str::<ClientMessage>(&msg)?;

    use ClientMessage::*;
    match msg {
        PressedA => Ok(()),
    }
}
 
//async fn request_match(id: &str, clients: &Clients, matchmaking: &Matchmaking) -> Result<()> {
//    let client = clients.lock().await.get(id).cloned().unwrap();
//    debug!("player {} requests match", client.player_name); 
//    matchmaking.lock().await.push_back(id.into());
//    Ok(())
//}
//
//async fn player_ready(id: &str, clients: &Clients, games: &Games) -> Result<()> {
//    let client = clients.lock().await.get(id).cloned().unwrap();
//    debug!("player {} ready", client.player_name); 
//    let mut games_lock = games.lock().await;
//    let gs = games_lock.get_mut(&client.game_id.ok_or(Error::NoGame("player not in a game".into()))?).ok_or(Error::NoGame("no such game".into()))?;
//    let ps = gs.player_state(&client.player_name)?;
//    ps.ready = true;
//    Ok(())
//}

pub async fn publish_handler(msg: ServerMessage, clients: Clients, params: PublishParams) -> WebResult<impl Reply> {
    clients
        .lock()
        .await
        .iter_mut()
        .filter(|(_, client)| if let Some(name) = &params.player_name { &client.player_name == name } else { true })
        .filter(|(_, client)| if params.game_id.is_some() { params.game_id == client.game_id } else { true })
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(serde_json::to_string(&msg).unwrap())));
            }
        });
    Ok(StatusCode::OK)
}
