use std::fmt::Display;

use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageData {
    pub user: String,
    pub message: String,
}

impl Display for MessageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts = Local::now().format("%H:%M");
        let MessageData { user, message, .. } = &self;
        write!(f, "\n{user} ({ts})\n{message}\n")
    }
}
