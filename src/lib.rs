use profiles::Profile;
use pelican_ui_std::Timestamp;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Local};
use uuid::Uuid;

pub mod components;
pub mod events;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub room_id: Uuid,
    pub profiles: Vec<Profile>,
    pub messages: Vec<Message>
}

impl Room {
    pub fn new(profiles: Vec<Profile>, messages: Vec<Message>) -> Self {
        Room { profiles, messages, room_id: Uuid::new_v4() }
    }

    pub fn from(profiles: Vec<Profile>) -> Self {
        Room { profiles, messages: vec![], room_id: Uuid::new_v4() }
    }

    pub fn add_message(&mut self, new: Message) {
        println!("Adding new message {:?} to room with id {:?}", new, self.room_id);
        self.messages.push(new);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    pub timestamp: Timestamp,
    pub author: Profile,
}

impl Message {
    pub fn from(message: String) -> Self {
        let author = Profile {
            user_name: "Marge Margarine".to_string(),
            biography: "Probably butter.".to_string(),
            identifier: "did::id::12345".to_string(),
            blocked_dids: Vec::new(),
        };

        let dt1 = "2025-05-19T08:12:45Z".parse::<DateTime<Utc>>().unwrap().with_timezone(&Local);

        Message {
            message,
            timestamp: Timestamp::new(dt1),
            author,
        }
    }
}