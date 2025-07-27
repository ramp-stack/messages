use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use profiles::service::Profiles;
use profiles::pages::{AccountActions};
use profiles::plugin::ProfilePlugin;
use pelican_ui::air::{Id};

use crate::components::{Cards, QuickDeselect, MessageType, ListItemMessages, TextMessageGroup, TextInputMessages, HeaderMessages};
use crate::events::{CreateMessageEvent, SetRoomEvent};
use crate::plugin::MessagesPlugin;
use crate::service::{RoomsRequest, Rooms, Message, PublicRooms};

use pelican_ui_std::{
    AppPage, Stack, Page,
    Header, IconButton, Text,
    ExpandableText, TextStyle, 
    Offset, ListItem, Content,
    Button, ButtonState, Searchbar,
    Bumper, TextInput, NavigateEvent, 
};

use uuid::Uuid;

// use crate::MSGPlugin;
// use crate::msg::{CurrentRoom, CurrentProfile};

#[derive(Component)] // () = Vec<(Id, Vec<OrangeName>, Vec<Message>)>
pub struct RoomsHome(Stack, Page, #[skip] Option<Id>, #[skip] AccountActions);

impl AppPage for RoomsHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(SelectRoomsRecipients::new(ctx, self.3))),
            1 => Ok(Box::new(RoomsMessage::new(ctx, self.2.unwrap(), self.3))),
            _ => Err(self),
        }
    }
}

impl std::fmt::Debug for RoomsHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoomsHome")
    }
}

impl RoomsHome {
    pub fn new(ctx: &mut Context, account_actions: AccountActions) -> Self {
        let search = IconButton::navigation(ctx, "search", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(2)));
        let header = Header::home(ctx, "Rooms", Some(search));
        let new_message = Button::primary(ctx, "Create Room", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let bumper = Bumper::single_button(ctx, new_message);
        // let rooms = ctx.state().get_or_default::<Rooms>().rooms();
        let rooms = PublicRooms::default();
        let cards = Cards::new(ctx, rooms.inner().clone(), 1);

        let content = Content::new(Offset::Start, vec![Box::new(cards) as Box<dyn Drawable>]);
        RoomsHome(Stack::center(), Page::new(Some(header), content, Some(bumper)), None, account_actions)
    }
}

impl OnEvent for RoomsHome {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            // let rooms = ctx.state().get_or_default::<Rooms>().clone().rooms();
            // if self.3 != rooms {
            //     if let Some(group) = self.1.content().find::<ListItemGroup>() {
            //         *group = ListItemRoomsMessages::new(ctx, rooms);
            //     } else {
            //         self.3 = rooms.clone();
            //         self.1.content().remove::<ExpandableText>();
            //         let group = ListItemRoomsMessages::new(ctx, rooms);
            //         self.1.content().items().push(Box::new(group));
            //         *self.1.content().offset() = Offset::Start;
            //     }
            // }
        } else if let Some(SetRoomEvent(id)) = event.downcast_ref::<SetRoomEvent>() {
            self.2 = Some(*id);
        }
        true
    }
}


#[derive(Component)]
pub struct SelectRoomsRecipients(Stack, Page, #[skip] ButtonState, #[skip] Option<Id>, #[skip] AccountActions, #[skip] Option<Uuid>, #[skip] bool);

impl AppPage for SelectRoomsRecipients {
    fn has_nav(&self) -> bool { false }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(RoomsHome::new(ctx, self.4))),
            // 1 => Ok(Box::new(RoomsMessage::new(ctx, self.3.unwrap(), self.4))),
            _ => Err(self),
        }
    }
}

impl std::fmt::Debug for SelectRoomsRecipients {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelectRoomsRecipients")
    }
}

impl SelectRoomsRecipients {
    pub fn new(ctx: &mut Context, account_actions: AccountActions) -> Self {
        let icon_button = None::<(&'static str, fn(&mut Context, &mut String))>;
        let searchbar = Searchbar::new(TextInput::new(ctx, None, None, "Profile name...", None, icon_button, false));

        let me = ProfilePlugin::me(ctx).0;
        let profiles = ctx.state().get_or_default::<Profiles>().clone().0;

        let recipients = profiles.keys().filter(|&orange_name| orange_name != &me).map(|orange_name| {
            ListItemMessages::recipient(ctx, orange_name)
        }).collect::<Vec<ListItem>>();

        // let recipients = vec![];
        
        let content = match recipients.is_empty() {
            true => {
                let text_size = ctx.theme.fonts.size.md;
                Box::new(Text::new(ctx, "No users found.", TextStyle::Secondary, text_size, Align::Center)) as Box<dyn Drawable>
            },
            false => Box::new(QuickDeselect::new(recipients)) as Box<dyn Drawable>
        };

        let content = Content::new(Offset::Start, vec![Box::new(searchbar), content]);
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let header = Header::stack(ctx, Some(back), "Create message", None);
        let button = Button::disabled(ctx, "Continue", move |ctx: &mut Context| ctx.trigger_event(CreateMessageEvent));

        let bumper = Bumper::single_button(ctx, button);
        SelectRoomsRecipients(Stack::center(), Page::new(Some(header), content, Some(bumper)), ButtonState::Default, None, account_actions, None, false)
    }
}

impl OnEvent for SelectRoomsRecipients {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let error = self.1.content().find::<QuickDeselect>().map(|deselect| deselect.get_orange_names().is_none()).unwrap_or(true);
            let error = error || self.6;
            let button = self.1.bumper().as_mut().unwrap().find::<Button>().unwrap();
            button.update_state(ctx, error, !error, &mut self.2);

            if let Some(uuid) = self.5 {
                // println!("ID was some {:?}", uuid);
                let mut guard = ctx.get::<MessagesPlugin>();
                let (plugin, ctx) = guard.get();
                let me = ProfilePlugin::me(ctx).0;
                if let Some((_, room)) = ctx.state().get::<Rooms>().expect("no rooms in state").0.iter().find(|(u, _)| *u == uuid) {
                    self.1.content().find::<QuickDeselect>().unwrap().get_orange_names().unwrap().iter().for_each(|on| {
                        plugin.request(RoomsRequest::Share(room.0, on.clone()));
                    });

                    let message = Message::invisible(me);
                    plugin.request(RoomsRequest::CreateMessage(room.0, message));
                    self.3 = Some(room.0);
                    ctx.trigger_event(NavigateEvent(1));
                }
            }
        } else if let Some(CreateMessageEvent) = event.downcast_ref::<CreateMessageEvent>() {
            self.6 = true;
            let mut guard = ctx.get::<MessagesPlugin>();
            let plugin = guard.get().0;
            let uuid = uuid::Uuid::new_v4();
            plugin.request(RoomsRequest::CreateRoom(uuid));
            self.5 = Some(uuid);
        }
        true
    }
}


#[derive(Component)]
pub struct RoomsMessage(Stack, Page, #[skip] Id, #[skip] AccountActions);

impl AppPage for RoomsMessage {
    fn has_nav(&self) -> bool { false }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(RoomsHome::new(ctx, self.3))),
            // 1 => Ok(Box::new(GroupInfo::new(ctx, self.2, self.3))),
            _ => Err(self),
        }
    }
}

impl std::fmt::Debug for RoomsMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoomsMessage")
    }
}

impl RoomsMessage {
    pub fn new(ctx: &mut Context, room_id: Id, account_actions: AccountActions) -> Self {
        let mut room = ctx.state().get_mut_or_default::<Rooms>().get(room_id).unwrap().clone();
        room.2.retain(|m| *m.message() != "__system__joined");
        let offset = if room.2.is_empty() {Offset::Center} else {Offset::End};
        let text_size = ctx.theme.fonts.size.md;
        let content = match room.2.is_empty() {
            true => Box::new(ExpandableText::new(ctx, "No messages yet.\nSend the first message.", TextStyle::Secondary, text_size, Align::Center, None)) as Box<dyn Drawable>,
            false => Box::new(TextMessageGroup::new(ctx, &room.2, MessageType::Rooms)) as Box<dyn Drawable>
        };
        let input = TextInputMessages::new(ctx, room.0);

        let bumper = Bumper::new(ctx, vec![Box::new(input)]);
        let content = Content::new(offset, vec![content]);
        let header = HeaderMessages::new(ctx, room.1.clone());
        RoomsMessage(Stack::center(), Page::new(Some(header), content, Some(bumper)), room_id, account_actions)
    }
}

impl OnEvent for RoomsMessage {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let mut room = ctx.state().get_mut_or_default::<Rooms>().get(self.2).unwrap().clone();
            room.2.retain(|m| *m.message() != "__system__joined");
            if !room.2.is_empty() {
                if let Some(group) = &mut self.1.content().find::<TextMessageGroup>() {
                    if room.2.len() > group.count() {
                        **group = TextMessageGroup::new(ctx, &room.2, MessageType::Rooms);
                    }
                } else {
                    self.1.content().remove::<ExpandableText>();
                    let group = Box::new(TextMessageGroup::new(ctx, &room.2, MessageType::Rooms)) as Box<dyn Drawable>;
                    self.1.content().items().push(group);
                    *self.1.content().offset() = Offset::End;
                }
            }
        }
        true
    }
}