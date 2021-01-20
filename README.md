# GGEZ Multiplayer Template

This implements a basic client / server multiplayer setup using `ggez` on the client and websocket communcation with a `warp` gameserver.

`server` app manages the gamestate, sends updates to clients, and handles inbound player input  
`client` app runs ggez event loop, handles inbound server updates, and sends input to the server  
`ggezmulti` is a shared library

## template
The main structs are in `ggezmulti` and `server/gamestate.rs`. Tailor these to suit your game and fill in logic in the `gametask` and  `player_action` functions on the server. 

On the client, you have to do all the usual ggez stuff as well as handle server updates and send input to the server. The client uses 2 auxilliary threads for server communication to avoid blocking the game thread.

## test
To test, run the server and then run the client. The client and server print all incoming messages. Hitting `A` on the keyboard in the client will send a message to the server, which will respond.
