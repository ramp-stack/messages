use pelican_ui::{Context, Plugin};
use pelican_ui::runtime;
use pelican_ui::air::Id;
// use serde_json::{Value, json};
// use std::hash::{DefaultHasher, Hasher, Hash};

use crate::service::{RoomsService, RoomsRequest, Message};

pub struct MessagesPlugin(runtime::Context);
impl Plugin for MessagesPlugin {
    fn new(ctx: &mut Context) -> Self {MessagesPlugin(ctx.runtime.clone())}
}
impl MessagesPlugin {
    pub fn request(&mut self, request: RoomsRequest) {
        self.0.send::<RoomsService>(&request)
    }

    pub fn create_message(ctx: &mut Context, id: Id, message: Message) {
        let mut guard = ctx.get::<MessagesPlugin>();
        let plugin = guard.get().0;
        plugin.request(RoomsRequest::CreateMessage(id, message));
    }
}
