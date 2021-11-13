# Rust Tic Tac Toe Server

A project for CS128H 2021

## Team Name

MK

## Team Member

Mark Zhang (zz91)

## Summary

A simple websocket game server written in rust for the game tic tac toe inspired by [Colyseus](https://github.com/colyseus/colyseus). The server will be using the actor concurrency model powered by [Lunatic](https://github.com/lunatic-solutions/lunatic) and the server code will be compiled to WASM for safer sandboxing. Each game room will be handled in a dedicated Lunatic process. State delta will be computed and sent to the client instead of the whole game state using [dipa](https://github.com/chinedufn/dipa).

## System Overview

- Server
  - Systems
    - Websocket layer
    - object delta
    - (de)serialization
    - Lunatic process spawning for each room
    - Message passing between processes
  - Server Logic
    - Room creation, deletion, listing
    - Game State Management (in game, gameover)
    - Game action handling

- Client
  - Connect to server
  - (de)serialization
  - display board
  - hanlde keyboard input


## Possible Challenges

- Lunatic is a relatively new runtime so getting it to work with websocket might take some time to figure out
- Handling state changes and serialization using [dipa](https://github.com/chinedufn/dipa)

## References

- [Lunatic](https://github.com/lunatic-solutions/lunatic) concurrency runtime
- [dipa](https://github.com/chinedufn/dipa) for struct deltas
- [Colyseus](https://github.com/colyseus/colyseus) (a node.js game server) for inspiration
