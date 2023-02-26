use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    select,
    sync::broadcast,
};
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().pretty().init();

    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    let addr = listener.local_addr().unwrap();

    info!("Server listening on {addr}");

    let (tx, _rx) = broadcast::channel(10);
    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        info!("Got connection request from {addr}");

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            info!("Client {addr} disconnected");
                            break;
                        }

                        info!("Client {addr} sent a message");

                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }

                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();
                        if addr != other_addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
