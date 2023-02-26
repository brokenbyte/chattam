use chattam::MessageData;

use cursive::{
    view::{Nameable, Resizable, ScrollStrategy, Scrollable},
    views::{
        Button, Dialog, DummyView, EditView, HideableView, LinearLayout, NamedView, ResizedView,
        ScrollView, TextView,
    },
    Cursive, View,
};
use cursive_aligned_view::Alignable;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

use crate::WRITE;

pub fn chat_window() -> impl View {
    // Holds the recieved messages
    let messages_list = LinearLayout::vertical()
        .with_name("messages")
        .full_width()
        .scrollable()
        .scroll_strategy(ScrollStrategy::StickToBottom)
        .on_scroll(|s, _r| {
            let mut messages = s
                .find_name::<ScrollView<ResizedView<NamedView<LinearLayout>>>>(
                    "messages_scrollable",
                )
                .unwrap();

            // If we've scrolled to the bottom, reset the "New message" alert
            if messages.is_at_bottom() {
                let alert = s.find_name::<HideableView<TextView>>("new_message");
                alert.unwrap().set_visible(false);

                // Need to reset the scroll strategy to make it stick to the bottom
                // as new messages come in again
                messages.set_scroll_strategy(ScrollStrategy::StickToBottom);
            }
        })
        .with_name("messages_scrollable");

    // Text box to type/send new messages
    let entrybox = EditView::new()
        .on_submit(on_submit)
        .with_name("new_message");

    let new_message_indicator = HideableView::new(TextView::new("New message!"))
        .visible(false)
        .with_name("new_message");

    let content = LinearLayout::vertical()
        .child(messages_list)
        .child(DummyView) // Add a gap between the text box and the messages
        .child(entrybox)
        .child(
            LinearLayout::horizontal()
                .child(new_message_indicator)
                .child(DummyView.full_width())
                .child(Button::new("Quit", |s| s.quit())),
        )
        .align_bottom_left()
        .fixed_height(20)
        .fixed_width(48);

    Dialog::new()
        .title("Chattam")
        .content(content)
        .fixed_width(50)
}

fn on_submit(s: &mut Cursive, msg: &str) {
    if msg.is_empty() {
        return;
    }

    let sender = s.cb_sink().to_owned();
    let message_body = MessageData {
        // TODO: Use a name entered by the user on startup
        user: "bob".into(),
        message: msg.to_string(),
    };

    // TODO: handle malformed JSON
    let msg_string = format!("{}\n", serde_json::to_string(&message_body).unwrap());

    tokio::spawn(async move {
        let mut writer = WRITE.get().unwrap().lock().await;
        let response = writer.write_all(msg_string.as_bytes()).await;
        info!("Sending a new message");

        let content = match response {
            Ok(_) => {
                info!("Successfully sent message");
                "".to_string()
            }
            Err(_) => {
                error!("Failed to send message");
                "Failed to send message!".to_string()
            }
        };

        sender
            .send(Box::new(|s| {
                let mut e = s.find_name::<EditView>("new_message").unwrap();
                e.set_content(content);
            }))
            .unwrap();
    });
}
