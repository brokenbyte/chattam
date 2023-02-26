use std::sync::Arc;

use chattam::MessageData;
use cursive::{
    views::{Dialog, HideableView, LinearLayout, NamedView, ResizedView, ScrollView, TextView},
    Cursive,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader, ReadHalf, WriteHalf},
    sync::{
        mpsc::{self, error::TryRecvError},
        Mutex, OnceCell,
    },
};
use tokio::{net::TcpStream, sync::mpsc::UnboundedSender};
use tracing::{error, info};

mod ui;
use ui::*;

static READ: OnceCell<Arc<Mutex<ReadHalf<TcpStream>>>> = OnceCell::const_new();
static WRITE: OnceCell<Arc<Mutex<WriteHalf<TcpStream>>>> = OnceCell::const_new();

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().pretty().init();

    let mut siv = cursive::default();
    siv.set_global_callback('q', |s| s.quit());

    let stream = TcpStream::connect("localhost:8080").await;

    if let Err(e) = stream {
        error!("Unable to connect to server: {e}");
        siv.add_layer(
            Dialog::new()
                .title("Chattam")
                .content(TextView::new("Unable to connect to server!"))
                .button("Quit", |s| s.quit()),
        );
        siv.run();
        std::process::exit(1);
    }

    let stream = stream.unwrap();

    let (read, write) = tokio::io::split(stream);
    READ.set(Arc::new(Mutex::new(read))).unwrap();
    WRITE.set(Arc::new(Mutex::new(write))).unwrap();

    // Channel for sending events from the thread listening to the server
    // to the thread controlling the UI
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Spawn a task for processing responses from the server
    read_from_server(tx);

    siv.add_layer(login::login_menu());

    let mut siv = siv.runner();
    let mut needs_refresh = false;

    // Start the event loop
    siv.refresh();
    loop {
        if !siv.is_running() {
            break;
        }

        match rx.try_recv() {
            Ok(m) => {
                siv.call_on_name("messages", |messages_list: &mut LinearLayout| {
                    let msg: MessageData = serde_json::from_str(&m).unwrap();

                    info!("Recieved a message");

                    messages_list.add_child(TextView::new(format!("{msg}")));
                });

                let messages = siv.find_name::<ScrollView<ResizedView<NamedView<LinearLayout>>>>(
                    "messages_scrollable",
                );

                if !messages.unwrap().is_at_bottom() {
                    let alert = siv.find_name::<HideableView<TextView>>("new_message");
                    alert.unwrap().set_visible(true);
                }

                needs_refresh = true;
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                siv.add_layer(
                    Dialog::new()
                        .content(TextView::new("Server disconnected!"))
                        .button("Quit", move |s: &mut Cursive| {
                            s.quit();
                        }),
                );
                siv.refresh();
            }
        }

        if needs_refresh {
            siv.refresh();
        }

        needs_refresh = siv.step();
    }
}

/// Waits for a message from the server in a separate thread and sends it over `tx`
fn read_from_server(tx: UnboundedSender<String>) {
    tokio::spawn(async move {
        let mut reader = READ.get().unwrap().lock().await;
        let mut reader = BufReader::new(&mut *reader);
        loop {
            let mut buffer = String::new();
            if reader.read_line(&mut buffer).await.unwrap() == 0 {
                info!("Read 0 bytes, shutting down...");
                return;
            };
            tx.send(buffer).unwrap();
        }
    });
}
