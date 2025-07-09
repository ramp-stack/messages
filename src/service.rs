use std::collections::BTreeMap;
use std::sync::LazyLock;
use std::time::Duration;

use maverick_os::Cache;
use pelican_ui::runtime::{Services, Service, ServiceList, ThreadContext, async_trait, self};
use pelican_ui::hardware;
use pelican_ui::State;
use pelican_ui::air::{OrangeName, Id, Service as AirService, Protocol, Validation, ChildrenValidation, HeaderInfo, RecordPath, Permissions};

use std::collections::HashSet;
use serde::{Serialize, Deserialize};
use chrono::{Utc, DateTime};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message(String, DateTime<Utc>, OrangeName, bool);
impl Message {
    pub fn from(message: String, author: OrangeName) -> Self {
        Message(message, Utc::now(), author, false)
    }

    pub fn invisible(author: OrangeName) -> Self {
        Message("__system__joined".to_string(), Utc::now(), author, true)
    }

    pub fn author(&self) -> &OrangeName {&self.2}
    pub fn timestamp(&self) -> &DateTime<Utc> {&self.1}
    pub fn message(&self) -> &String {&self.0}
    pub fn is_read(&self) -> &bool {&self.3}
    pub fn read(&mut self, status: bool) {self.3 = status}
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Rooms(pub Vec<(Uuid, Room)>);

impl Rooms {
    pub fn rooms(&self) -> Vec<Room> {
        self.0.clone().into_iter().map(|(_, r)| r).collect()
    }
    pub fn get(&mut self, id: Id) -> Option<&mut Room> {
        self.0.iter_mut().find(|(_, i)| *i.0 == *id).map(|(_, r)| r)
    }
}

pub type Room = (Id, Vec<OrangeName>, Vec<Message>);

static ROOMS: LazyLock<Id> = LazyLock::new(|| Id::hash(&"RoomsV1".to_string()));
static MESSAGES: LazyLock<Id> = LazyLock::new(|| Id::hash(&"MessagesV1".to_string()));

const ROOMS_PERMISSIONS: Permissions = Permissions::new(Some((true, true)), None, BTreeMap::new());
const MESSAGES_PERMISSIONS: Permissions = Permissions::new(None, None, BTreeMap::new());

static ROOMS_PROTOCOL: LazyLock<Protocol> = LazyLock::new(|| {
    let cv = ChildrenValidation::new(vec![*MESSAGES], true, true, false);
    let validation = Validation::new(Some(cv), None, BTreeMap::new(), false);
    let header = HeaderInfo::new(None, BTreeMap::new(), Vec::new());
    Protocol::new(validation, header, *ROOMS)
});

static MESSAGES_PROTOCOL: LazyLock<Protocol> = LazyLock::new(|| {
    let validation = Validation::new(None, None, BTreeMap::new(), false);
    let header = HeaderInfo::new(None, BTreeMap::new(), Vec::new());
    Protocol::new(validation, header, *MESSAGES)
});

#[derive(Serialize, Deserialize, Debug)]
pub enum RoomsRequest {
    CreateRoom(Uuid),
    CreateMessage(Id, Message),
    Share(Id, OrangeName),
}

#[derive(Debug)]
pub struct RoomsService{
}

impl Services for RoomsService {
    fn services() -> ServiceList {
        let mut services = ServiceList::default();
        services.insert::<RoomsSync>();
        services
    }
}

#[async_trait]
impl Service for RoomsService {
    type Send = ();
    type Receive = RoomsRequest;

    async fn new(_hardware: &mut hardware::Context) -> Self {
        RoomsService{
        }
    }

    async fn run(&mut self, ctx: &mut ThreadContext<Self::Send, Self::Receive>) -> Result<Option<Duration>, runtime::Error> {
        let mut cache = RoomsCache::from_cache(&mut ctx.hardware.cache).await;
        while let Some((_, request)) = ctx.get_request() {
            match request {
                RoomsRequest::CreateRoom(uuid) => {
                    while let (_, Some(_)) = AirService::create_private(ctx, RecordPath::root(), ROOMS_PROTOCOL.clone(), cache.rooms_idx, ROOMS_PERMISSIONS, serde_json::to_vec(&uuid)?).await? {
                        cache.rooms_idx += 1;
                    }
                },
                RoomsRequest::CreateMessage(room, message) => {
                    let mut x = cache.rooms.get(&RecordPath::root().join(room)).unwrap().2;
                    while let (_, Some(_)) = AirService::create_private(ctx, RecordPath::root().join(room), MESSAGES_PROTOCOL.clone(), x, MESSAGES_PERMISSIONS, serde_json::to_vec(&message)?).await? {
                        x += 1;
                    }
                },
                RoomsRequest::Share(room, name) => {
                    let message = Message::invisible(name.clone());
                    let path = RecordPath::root().join(room);
                    AirService::share(ctx, name, ROOMS_PERMISSIONS, path).await?;
                    let mut x = cache.rooms.get(&RecordPath::root().join(room)).unwrap().2;
                    while let (_, Some(_)) = AirService::create_private(ctx, RecordPath::root().join(room), MESSAGES_PROTOCOL.clone(), x, MESSAGES_PERMISSIONS, serde_json::to_vec(&message)?).await? {
                        x += 1;
                    }
                },
            }
        }

        Ok(Some(Duration::from_millis(16)))
    }

    fn callback(_state: &mut State, _response: Self::Send) {
        // let mut rooms = state.get::<Rooms>().0;
        // // if response.2 {state.set(&Name(Some(response.0.clone())));}
        // rooms.insert(response.0, response.1);
        // state.set(&Rooms(rooms));
    }
}

#[derive(Debug)]
pub struct RoomsSync{
    cache: RoomsCache,
    init: bool 
}

impl Services for RoomsSync {}

#[async_trait]
impl Service for RoomsSync {
    type Send = Vec<(Uuid, Room)>;
    type Receive = ();

    async fn new(hardware: &mut hardware::Context) -> Self {
        RoomsSync{
            cache: RoomsCache::from_cache(&mut hardware.cache).await,
            init: false
        }
    }

    async fn run(&mut self, ctx: &mut ThreadContext<Self::Send, Self::Receive>) -> Result<Option<Duration>, runtime::Error> {
        let mut mutated = false;
        println!("running {:?}", self.cache.rooms_idx);

        for (_, path) in AirService::receive(ctx, self.cache.datetime).await?.into_iter() {
            // let uuid: Uuid = serde_json::from_slice(&AirService::read_private(ctx, path.clone()).await?.unwrap().0.payload).unwrap();
            // self.cache.rooms.entry(path).or_insert((uuid, vec![], 0));
            // mutated = true;

            println!("Creating pointer.");
            let mut x = self.cache.rooms_idx;
            while let (_, Some(_)) = AirService::create_pointer(ctx, RecordPath::root(), path.clone(), x).await? {
                x += 1;
            }
            println!("Done creating pointers.");
            mutated = true;
        }

        println!("Done receiving.");

        self.cache.datetime = chrono::Utc::now();

        while let (path, Some(_)) = AirService::discover(ctx, RecordPath::root(), self.cache.rooms_idx, vec![ROOMS_PROTOCOL.clone()]).await? {
            println!("Discovering...");
            if let Some(path) = path {
                if let Ok(uuid) = serde_json::from_slice(&AirService::read_private(ctx, path.clone()).await?.unwrap().0.payload) {
                    println!("Uuid: {:?}...", uuid);
                    self.cache.rooms.entry(path).or_insert((uuid, vec![], 0));
                    mutated = true;
                } else {println!("_--- ROOM HAD NO UUID ---_");}
            }
            self.cache.rooms_idx += 1;
        }
        println!("Done discovering.");

        for (room, (_, messages, index)) in &mut self.cache.rooms {
            while let (path, Some(_)) = AirService::discover(ctx, room.clone(), *index, vec![MESSAGES_PROTOCOL.clone()]).await? {
                if let Some(path) = path {
                    if let Ok(message) = serde_json::from_slice(&AirService::read_private(ctx, path).await?.unwrap().0.payload) {
                        messages.insert(*index as usize, message);
                        mutated = true;
                    }
                }
                *index += 1;
            }
        }

        println!("Done messages.");
        
        if mutated || !self.init {
            self.init = true;
            ctx.callback(self.cache.rooms.iter().map(|(p, (u, m, _))| {
                let authors: Vec<_> = m.iter().map(|Message(_, _, a, _)| a.clone()).collect::<HashSet<_>>().into_iter().collect();
                (*u, (p.last(), authors, m.clone()))
            }).collect());
            println!("Callback done.");
        }

        println!("Done updating.");
        self.cache.cache(&mut ctx.hardware.cache).await;
        println!("DONE");
        Ok(Some(Duration::from_secs(1)))
    }

    fn callback(state: &mut State, response: Self::Send) {
        println!("Callback...");
        state.set(Rooms(response))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RoomsCache {
    pub rooms_idx: u32,
    pub rooms: BTreeMap<RecordPath, (Uuid, Vec<Message>, u32)>,
    pub datetime: DateTime<Utc>,
}

impl RoomsCache {
    pub async fn cache(&self, cache: &mut Cache) {
        // let other = cache.get::<Cache>("RoomCache").await;
        cache.set("RoomCache", self).await;
    }

    pub async fn from_cache(cache: &mut Cache) -> Self {
        cache.get("RoomCache").await
    }

    // pub fn merge(self, other: Self) -> Self {
    //     Cache {
    //         rooms_idx: self.rooms_idx.max(other.rooms_idx),
    //         rooms: self.rooms.extend(other.)
    //     }
    // }
}

impl Default for RoomsCache {
    fn default() -> Self {
        println!("RoomsCache as default");
        RoomsCache {
            rooms_idx: 0,
            rooms: BTreeMap::new(),
            datetime: DateTime::UNIX_EPOCH,
        }
    }
}

// pub struct PublicRoom(String, String, AvatarContent, Vec<OrangeName>, Vec<Message>); // title, subtitle, members, messages
// impl PublicRoom {
//     pub fn new(t: &str, s: &str, p: Option<resources::Image>) -> Self {
//         let avatar = AvatarContentMessages::rooms(p);
//         PublicRoom(t.to_string(), s.to_string(), avatar, Vec::new(), Vec::new())
//     }

//     pub fn title(&mut self) -> &mut String {&mut self.0}
//     pub fn subtitle(&mut self) -> &mut tring {&mut self.1}
//     pub fn avatar(&mut self) -> &mut AvatarContent {&mut self.2}
//     pub fn members(&self) -> &Vec<OrangeName> {&self.3}
//     pub fn messages(&self) -> &Vec<Message> {&self.4}
// }

// pub struct PublicRooms(Vec<PublicRoom>);

// impl PublicRooms {
//     pub fn inner(&mut self) -> &mut Vec<PublicRoom> {
//         &mut self.0
//     }
// }

// impl Default for PublicRooms {
//     fn default() -> Self {
//         let rooms = vec![
//             PublicRoom::new("the orange room", "a room for all things orange", None)
//         ];

//         PublicRooms(rooms)
//     }
// }