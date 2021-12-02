# How to Build and Run This Project

### Install the [Lunatic](https://github.com/lunatic-solutions/lunatic) runtime

- You can install with homebrew or build from source

### Build this project

1. Clone this project

   ```bash
   git clone git@github.com:MarkintoshZ/rust-tic-tac-toe-server.git
   cd rust-tic-tac-toe-server
   ```

2. Build and start the server

   ```bash
   cd server
   cargo start --release
   ```

3. Build and start the client

   ```bash
   cd ..
   cargo start --release --bin client
   ```

   Commands for the client are as follows:

   - join server
   - leave server
   - create room
   - join room
   - leave room
   - place at \_ \_: for placing node on the board (e.g. "place at 0 0" will place a node at the top left corner of the board)
