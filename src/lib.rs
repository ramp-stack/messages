use profiles::Profile;
use pelican_ui_std::Timestamp;

use serde::{Serialize, Deserialize};

pub mod components;
pub mod events;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub profiles: Vec<Profile>,
    pub messages: Vec<Message>
}

impl Room {
    pub fn new(profiles: Vec<Profile>, messages: Vec<Message>) -> Self {
        Room { profiles, messages }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    pub timestamp: Timestamp,
    pub author: Profile,
}
