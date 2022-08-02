# Rust Tic Tac Toe Server

A project for CS128H 2021

### Video Demo (Don't mind the clickbait thumbnail)
[![Video Demo](https://i9.ytimg.com/vi/4xsORjs0w3U/maxresdefault.jpg?time=1659464700000&sqp=CPzXpZcG&rs=AOn4CLDnRRFEE8cMkHw23wiiTWLVU9fJPg)](https://www.youtube.com/watch?v=4xsORjs0w3U)

## Team Name

MK

## Team Member

Mark Zhang (zz91)

## Summary

A simple multiplayer game server written in rust for the game tic tac toe inspired by [Colyseus](https://github.com/colyseus/colyseus). The server uses the actor concurrency model powered by [Lunatic](https://github.com/lunatic-solutions/lunatic) and the server code will be compiled to WASM for safer sandboxing. Each game room will be handled in a dedicated Lunatic process. State delta will be computed and sent to the client instead of the whole game state using [dipa](https://github.com/chinedufn/dipa).

## System Overview

- Server
  - Systems
    - object delta (dipa)
    - (de)serialization (bincode)
    - Lunatic process spawning for each room
    - Message passing between processes
  - Server Logic
    - Room creation, deletion
    - Game State Management (in game, gameover)
    - Game action handling

- Client
  - Connect to server
  - (de)serialization
  - display board
  - handle keyboard input


## Possible Challenges

- Lunatic is a relatively new runtime so getting it to work with websocket might take some time to figure out
- Handling state changes and serialization using [dipa](https://github.com/chinedufn/dipa)

## References

- [Lunatic](https://github.com/lunatic-solutions/lunatic) concurrency runtime
- [Lunatic.Chat](https://github.com/lunatic-solutions/chat) Lunatic demo chat app
- [dipa](https://github.com/chinedufn/dipa) library for computing and applying object deltas
- [Colyseus](https://github.com/colyseus/colyseus) (a node.js game server) for inspiration

## License
MIT

### Feel free to 🌟 to show some love!
