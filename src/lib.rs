use pelican_ui::prelude::*;
use pelican_ui_profiles::Profile;

use serde::{Serialize, Deserialize};

mod components;
mod events;

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

pub mod prelude {
    pub use crate::{Room, Message};
    pub use crate::components::*;
    pub use crate::events::*;
}

