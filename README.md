# ggez multiplayer starter

This is a basic client / server multiplayer setup using [ggez](https://github.com/ggez/ggez) on the client and websocket communication with a [warp](https://github.com/seanmonstar/warp) server.

* `server` app manages the gamestate, sends updates to clients, and handles inbound player actions  
* `client` app runs the local game loop, handles inbound server updates, and sends actions to the server  
* `ggezmulti` is a shared library

## where to start
The main structs are in `ggezmulti` and `server/gamestate`. Tailor these to suit your purposes and fill in logic in the `gametask` and  `player_action` functions on the server. 

On the client, you have to do all the usual ggez stuff as well as handle server updates and send actions to the server. The client `update` function shows one way.

## test
```
# Run the server  
$ cd ggez-multiplayer  
$ cargo run --manifest-path server/Cargo.toml

# Then run the client passing player name and game id  
$ cargo run --manifest-path client/Cargo.toml -- "Player One" "New Game" 
```
The client and server print all incoming messages. Hit `A` on the keyboard in the game client to send a message to the server. Check it was received by the server and look for its response.
