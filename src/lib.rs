use profiles::service::{Profile, Profiles};
use maverick_os::air::air;
use air::orange_name::{OrangeName, OrangeSecret};

use pelican_ui_std::Timestamp;
use pelican_ui::Context;

use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Local};
use uuid::Uuid;

pub mod components;
pub mod events;
pub mod pages;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    pub authors: Vec<OrangeName>,
    pub messages: Vec<Message>
}

impl Room {
    pub fn new(authors: Vec<OrangeName>) -> Self {
        println!("Made new room");
        Room { authors, messages: vec![] }
    }

    pub fn add_message(&mut self, new: Message) {
        self.messages.push(new);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
    pub timestamp: Timestamp,
    pub author: OrangeName,
}

impl Message {
    pub fn from(message: String, author: OrangeName) -> Self {
        Message {
            message,
            timestamp: Timestamp::new(Local::now()),
            author,
        }
    }
}


#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Rooms(pub BTreeMap<uuid::Uuid, Room>);

impl Rooms {
    pub fn new(ctx: &mut Context) -> Self {
        println!("MAKING NEW ROOMS");
        Rooms(fake_rooms(ctx))
    }

    pub fn add(&mut self, room: Room, id: uuid::Uuid) {
        self.0.insert(id, room);
    }
}

// #[derive(Serialize, Deserialize, Default, Clone, Debug)]
// pub struct AllProfiles(pub Vec<(OrangeName)>);

// impl AllProfiles {
//     pub fn new(ctx: &mut context) -> Self {
//         AllProfiles(fake_profiles(ctx))
//     }
// }

fn fake_rooms(ctx: &mut Context) -> BTreeMap<Uuid, Room> {
    let dt1 = "2025-05-19T08:12:45Z".parse::<DateTime<Utc>>().unwrap().with_timezone(&Local);
    let testers = fake_profiles(ctx); 
    let a = testers[0].0.clone();
    let b = testers[1].0.clone();
    let c = testers[2].0.clone();

    let test_rooms_data = vec![
        Room {
            authors: vec![a.clone()],
            messages: vec![
                Message {
                    message: "hey".to_string(),
                    timestamp: Timestamp::new(dt1),
                    author: a.clone(),
                },
                Message {
                    message: "hello??".to_string(),
                    timestamp: Timestamp::new(dt1),
                    author: a.clone(),
                },
            ],
        },
        Room {
            authors: vec![a.clone(), b.clone()],
            messages: vec![
                Message {
                    message: "you there?".to_string(),
                    timestamp: Timestamp::new(dt1),
                    author: a.clone(),
                },
                Message {
                    message: "yeah, why?".to_string(),
                    timestamp: Timestamp::new(dt1),
                    author: b.clone(),
                },
            ],
        },
        Room {
            authors: vec![c.clone()],
            messages: vec![
                Message {
                    message: "been wanting to say...".to_string(),
                    timestamp: Timestamp::new(dt1),
                    author: c.clone(),
                },
                Message {
                    message: "how u beeeeen??".to_string(),
                    timestamp: Timestamp::new(dt1),
                    author: c.clone(),
                },
            ],
        },
    ];

    let test_rooms: BTreeMap<_, _> = test_rooms_data.into_iter().map(|room| (uuid::Uuid::new_v4(), room)).collect();

    println!("Returning test rooms :{:?}", test_rooms);

    test_rooms

}


pub fn fake_profiles(ctx: &mut Context) -> Vec<(OrangeName, Profile)> {
    let testers = vec![
        (new_orange_name(), BTreeMap::from([
            ("username".to_string(), "SaltyWeasel".to_string()),
            ("biography".to_string(), "Only calm when microwaving ice cream.".to_string()),
        ])),
        (new_orange_name(), BTreeMap::from([
            ("username".to_string(), "ShakyDuck".to_string()),
            ("biography".to_string(), "Emotionally unstable. Surprisingly good at taxes.".to_string()),
        ])),
        (new_orange_name(), BTreeMap::from([
            ("username".to_string(), "SnappyDolphin".to_string()),
            ("biography".to_string(), "Accidentally joined a cult. Stayed for snacks.".to_string()),
        ]))
    ];

    let mut profiles = ctx.state().get::<Profiles>().0;
    profiles.insert(testers[0].0.clone(), testers[0].1.clone());
    profiles.insert(testers[1].0.clone(), testers[1].1.clone());
    profiles.insert(testers[2].0.clone(), testers[2].1.clone());

    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "FluffyNumbat".to_string()),
        ("biography".to_string(), "Accidentally started cult. Now taking applications.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "DustyKinkajou".to_string()),
        ("biography".to_string(), "Certified wizard. Lost wand. Still dangerous.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "WigglyOkapi".to_string()),
        ("biography".to_string(), "Allergic to boredom and normal jobs.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "BouncyPangolin".to_string()),
        ("biography".to_string(), "Can’t cook. Can haunt your dreams.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "ClumsyRacoon".to_string()),
        ("biography".to_string(), "Owns zero goats. Deeply regrets everything.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "NervousOstritch".to_string()),
        ("biography".to_string(), "Eats stress for breakfast. Needs dessert.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "GreedyLlama".to_string()),
        ("biography".to_string(), "Was normal. Then touched a toad.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "DustyKinkajou".to_string()),
        ("biography".to_string(), "Haunted by snacks I never ate.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "LoopyFossa".to_string()),
        ("biography".to_string(), "Mildly feral. Surprisingly employable. Occasionally sparkly.".to_string()),
    ]));
    profiles.insert(new_orange_name(), BTreeMap::from([
        ("username".to_string(), "SpottedSloth".to_string()),
        ("biography".to_string(), "Talks to plants. They talk back. That's a lie.".to_string()),
    ]));

    ctx.state().set(&Profiles(profiles));

    testers
}

fn new_orange_name() -> OrangeName {
    OrangeSecret::new().name()
}