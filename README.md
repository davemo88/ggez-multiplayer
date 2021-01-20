# ggez Multiplayer Template

This implements a basic client / server multiplayer setup using `ggez` on the client and websocket communcation with a `warp` server.

* `server` app manages the gamestate, sends updates to clients, and handles inbound player input  
* `client` app runs the local game loop, handles inbound server updates, and sends input to the server  
* `ggezmulti` is a shared library

## where to start
The main structs are in `ggezmulti` and `server/gamestate.rs`. Tailor these to suit your game and fill in logic in the `gametask` and  `player_action` functions on the server. 

On the client, you have to do all the usual ggez stuff as well as handle server updates and send input to the server. The client `update` function shows one way to do it.

## test
```
# Run the server  
$ cd ggez-multiplayer  
$ cargo run --manifest-path server/Cargo.toml

# Then run the client passing `player_name` and `game_id`  
$ cargo run --manifest-path client/Cargo.toml -- "Player One" "New Game" 
```
The client and server print all incoming messages. Hit `A` on the keyboard in the game client to send a message to the server. Check it was received by the server and look for its response.
