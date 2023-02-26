# Chattam
Chattam is a toy project to play with networking, async, and channels. It consists of a client and server
which communicate by sending JSON over a TCP connection.

The server spawns a new task for each incoming connection, and sends the message it receives to all other
connected clients over a tokio `broadcast` channel.

The TUI client spawns a task to listen for new messages from the server and sends them over a channel to the
main thread to update the displayed messages and any other state.

## Running
The server is very straightforward to start; simply run `cargo run --bin server` in a terminal and
it will run and print out it's listening address.

The client is similar, but because it logs to `stdout` you'll need to redirect it to a file or it will
write over the UI. With the server running, in a separate terminal run `cargo run --bin tui 1>/some/file`
to redirect the logs to `/some/file`; I like to `tail -f` the logs while running to view them.
Run another client in a 3rd terminal window to begin messaging back and forth.


### To-do:
- Add ability to create/join different channels
- Add authentication
- Add message persistence (probably Postgres)
- Configurable server address for client
- Handle multiple clients better instead of using a `broadcast::channel()`
- Show messages you've sent as the client
- Better error handling instead of `.unwrap()`
- Add some GIFs to the README
- Create a GUI client (probably `egui`)
- Make the TUI client log to a file directly instead of `stdout`
- Actually accept and use a username from the client
