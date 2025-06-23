use std::collections::{BTreeSet, BTreeMap};
use std::sync::LazyLock;
use std::time::Duration;

use maverick_os::Cache;

use pelican_ui::runtime::{Services, Service, ServiceList, ThreadContext, async_trait, self};
use pelican_ui::hardware;
use pelican_ui::State;
use pelican_ui::air::{OrangeName, Id, PublicItem, Filter, Request, Service as AirService, Protocol, Validation, ChildrenValidation, HeaderInfo, RecordPath, Permissions};
use pelican_ui_std::Timestamp;

use serde::{Serialize, Deserialize};
use chrono::Local;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message(String, Timestamp, OrangeName);
impl Message {
    pub fn from(message: String, author: OrangeName) -> Self {
        Message(message, Timestamp::new(Local::now()), author)
    }

    pub fn author(&self) -> &OrangeName {&self.2}
    pub fn timestamp(&self) -> &Timestamp {&self.1}
    pub fn message(&self) -> &String {&self.0}
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Rooms(pub Vec<Room>);

pub type Room = (Id, Vec<OrangeName>, Vec<Message>);

static ROOMS: LazyLock<Id> = LazyLock::new(|| Id::hash(&"RoomsV1".to_string()));
static MESSAGES: LazyLock<Id> = LazyLock::new(|| Id::hash(&"MessagesV1".to_string()));

const ROOMS_PERMISSIONS: Permissions = Permissions::new(Some((true, true)), None, BTreeMap::new());

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
    CreateRoom,
    CreateMessage(Id, Message),
    // InsertField(String, String),
    // RemoveField(String),
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
            println!("Processing Request {:?}", request);
            match request {
                RoomsRequest::CreateRoom => {
                    println!("Try to create room");
                    while let (path, Some(_)) = AirService::create_private(ctx, RecordPath::root(), ROOMS_PROTOCOL.clone(), cache.rooms_idx, ROOMS_PERMISSIONS, vec![]).await? {
                        cache.rooms_idx += 1;
                    }
                    println!("Room successfully created.");
                },
                RoomsRequest::CreateMessage(room, message) => {
                    let mut x = *cache.rooms.get(&RecordPath::root().join(room)).unwrap();
                    while let (path, Some(_)) = AirService::create_private(ctx, RecordPath::root().join(room), MESSAGES_PROTOCOL.clone(), x, ROOMS_PERMISSIONS, serde_json::to_vec(&message)?).await? {
                        x += 1;
                    }
                }
            }
        }

        Ok(Some(Duration::from_millis(16)))
    }

    fn callback(state: &mut State, response: Self::Send) {
        // let mut rooms = state.get::<Rooms>().0;
        // // if response.2 {state.set(&Name(Some(response.0.clone())));}
        // rooms.insert(response.0, response.1);
        // state.set(&Rooms(rooms));
    }
}

#[derive(Debug)]
pub struct RoomsSync{
    cache: RoomsCache,
}

impl Services for RoomsSync {}

#[async_trait]
impl Service for RoomsSync {
    type Send = Vec<Room>;
    type Receive = ();

    async fn new(hardware: &mut hardware::Context) -> Self {
        RoomsSync{
            cache: RoomsCache::from_cache(&mut hardware.cache).await,
        }
    }

    async fn run(&mut self, ctx: &mut ThreadContext<Self::Send, Self::Receive>) -> Result<Option<Duration>, runtime::Error> {
        let mut mutated = false;
        while let Some((path, time)) = AirService::discover(ctx, RecordPath::root(), self.cache.rooms_idx, vec![ROOMS_PROTOCOL.clone()]).await? {
            self.cache.rooms.entry(path).or_insert(0);
            self.cache.rooms_idx += 1;
            mutated = true;
        }
        if mutated {ctx.callback(self.cache.rooms.iter().map(|(p, _)| (p.last(), vec![], vec![])).collect())}
        self.cache.cache(&mut ctx.hardware.cache).await;
        Ok(Some(Duration::from_secs(1)))
    }

    fn callback(state: &mut State, response: Self::Send) {
        state.set(Rooms(response))
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct RoomsCache {
    pub rooms_idx: u32,
    pub rooms: BTreeMap<RecordPath, u32>,
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