use cursive::{
    views::{Dialog, TextView},
    Cursive,
};

use crate::ui::chat::chat_window;

pub fn login_menu() -> Dialog {
    Dialog::new()
        .title("Chattam")
        .content(TextView::new("Enter your name:"))
        .button("Okay", move |s: &mut Cursive| {
            s.pop_layer();
            s.add_layer(chat_window());
        })
}
