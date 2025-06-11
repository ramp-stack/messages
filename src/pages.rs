use pelican_ui::events::{Event, OnEvent, Key, NamedKey, KeyboardState, KeyboardEvent, TickEvent};
use pelican_ui::drawable::{Drawable, Component, Align, Span, Image};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use profiles::service::{Profile, Profiles, Name};
use profiles::components::AvatarContentProfiles;
use profiles::pages::{Account, UserAccount};
use profiles::plugin::ProfileHelper;

use crate::{Room, Rooms, Message};
use crate::components::{QuickDeselect, TextMessage, MessageType, ListItemMessages, TextMessageGroup, TextInputMessages, HeaderMessages};
use crate::events::CreateMessageEvent;

use pelican_ui_std::{
    AppPage, Stack, Page,
    Header, IconButton,
    Avatar, AvatarContent,
    AvatarIconStyle, Icon, Text,
    ExpandableText, ClearActiveInput,
    TextStyle, Content,
    Offset, ListItem,
    Button, ButtonState,
    Bumper, TextInput, Alert,
    SetActiveInput, IS_MOBILE,
    QuickActions, ListItemSelector,
    NavigateEvent, DataItem,
    Timestamp, ListItemGroup,
};

use::chrono::{DateTime, Local, Utc};
use std::sync::{Arc, Mutex};

// use crate::MSGPlugin;
// use crate::msg::{CurrentRoom, CurrentProfile};

#[derive(Debug, Component, AppPage)]
pub struct MessagesHome(Stack, Page);
impl OnEvent for MessagesHome {}

impl MessagesHome {
    pub fn new(ctx: &mut Context) -> (Self, bool) {
        let header = Header::home(ctx, "Messages");
        let new_message = Button::primary(ctx, "New Message", |ctx: &mut Context| {
            let page = SelectRecipients::new(ctx);
            ctx.trigger_event(NavigateEvent::new(page))
        });

        let bumper = Bumper::single_button(ctx, new_message);
        let rooms = ctx.state().get::<Rooms>().0;
        // println!("rooms {:?}", rooms);
        let messages = rooms.into_iter().map(|(id, room)| {
            match room.authors.len() > 1 {
                true => {
                    ListItemMessages::group_message(ctx, &id, move |ctx: &mut Context| {
                        let page = GroupMessage::new(ctx, &id);
                        ctx.trigger_event(NavigateEvent::new(page));
                    })
                },
                false => {
                    ListItemMessages::direct_message(ctx, &id, move |ctx: &mut Context| {
                        let (on_return, with_nav) = Self::new(ctx);
                        let page = DirectMessage::new(ctx, &id, (on_return.into_boxed(), with_nav));
                        ctx.trigger_event(NavigateEvent::new(page));
                    })
                }
            }
        }).collect::<Vec<ListItem>>();
        let text_size = ctx.theme.fonts.size.md;
        let instructions = Text::new(ctx, "No messages yet.\nGet started by messaging a friend.", TextStyle::Secondary, text_size, Align::Center);

        let content = match !messages.is_empty() {
            true => Content::new(Offset::Start, vec![Box::new(ListItemGroup::new(messages))]),
            false => Content::new(Offset::Center, vec![Box::new(instructions)])
        };

        (MessagesHome(Stack::center(), Page::new(header, content, Some(bumper))), true)
    }
}

#[derive(Debug, Component, AppPage)]
pub struct SelectRecipients(Stack, Page, #[skip] ButtonState);

impl SelectRecipients {
    pub fn new(ctx: &mut Context) -> (Self, bool) {
        let icon_button = None::<(&'static str, fn(&mut Context, &mut String))>;
        let searchbar = TextInput::new(ctx, None, None, "Profile name...", None, icon_button);

        let profiles = ctx.state().get::<Profiles>().0;

        let recipients = profiles.iter().map(|(orange_name, profile)| {
            ListItemMessages::recipient(ctx, orange_name)
        }).collect::<Vec<ListItem>>();
        
        let content = recipients.is_empty().then(|| {
            let text_size = ctx.theme.fonts.size.md;
            Box::new(Text::new(ctx, "No users found.", TextStyle::Secondary, text_size, Align::Center)) as Box<dyn Drawable>
        }).unwrap_or_else(|| {
            Box::new(QuickDeselect::new(recipients)) as Box<dyn Drawable>
        });

        let content = Content::new(Offset::Start, vec![Box::new(searchbar), content]);
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| {
            let page = MessagesHome::new(ctx);
            ctx.trigger_event(NavigateEvent::new(page))
        });

        let header = Header::stack(ctx, Some(back), "Send to contact", None);
        let button = Button::disabled(ctx, "Continue", move |ctx: &mut Context| ctx.trigger_event(CreateMessageEvent));

        let bumper = Bumper::single_button(ctx, button);
        (SelectRecipients(Stack::center(), Page::new(header, content, Some(bumper)), ButtonState::Default), false)
    }
}

impl OnEvent for SelectRecipients {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let error = self.1.content().find::<QuickDeselect>().unwrap().get_orange_names().is_none();
            let button = self.1.bumper().as_mut().unwrap().find::<Button>().unwrap();
            button.update_state(ctx, error, !error, &mut self.2);
        } if let Some(CreateMessageEvent) = event.downcast_ref::<CreateMessageEvent>() {
            let id = uuid::Uuid::new_v4();
            let orange_names = self.1.content().find::<QuickDeselect>().unwrap().get_orange_names().unwrap();
            let mut rooms = ctx.state().get::<Rooms>();

            if orange_names.len() == 1 {
                for (id, room) in rooms.0.iter() {
                    if room.authors.len() == 1 && room.authors[0] == orange_names[0] {
                        let (on_return, with_nav) = MessagesHome::new(ctx);
                        let page = DirectMessage::new(ctx, id, (on_return.into_boxed(), with_nav));
                        ctx.trigger_event(NavigateEvent::new(page));
                        return true;
                    }
                }
            }

            println!("create dm");
            let (page, with_nav) = match orange_names.len() > 1 { 
                true => {
                    let (page, with_nav) = GroupMessage::new(ctx, &id);
                    (page.into_boxed(), with_nav) 
                },
                false => { 
                    let (on_return, with_nav) = MessagesHome::new(ctx);
                    let (page, with_nav) = DirectMessage::new(ctx, &id, (on_return.into_boxed(), with_nav));
                    (page.into_boxed(), with_nav)
                }
            };

            rooms.add(Room::new(orange_names.clone()), id);
            ctx.state().set(&rooms);
            ctx.trigger_event(NavigateEvent(Some(page), with_nav));
        }
        true
    }
}

#[derive(Debug, Component, AppPage)]
pub struct DirectMessage(Stack, Page, #[skip] uuid::Uuid);

impl DirectMessage {
    pub fn new(ctx: &mut Context, room_id: &uuid::Uuid, account_return: (Box<dyn AppPage>, bool)) -> (Self, bool) {
        let rooms = ctx.state().get::<Rooms>();
        let room = rooms.0.get(room_id).unwrap();

        let profiles = ctx.state().get::<Profiles>();
        let orange_name = room.authors[0].clone();

        let user = profiles.0.get(&orange_name).unwrap();
        let username = user.get("username").unwrap();

        let my_orange_name = ProfileHelper::get_my_profile(ctx).0;
        let is_blocked = ProfileHelper::has_blocked(ctx, &my_orange_name, &orange_name);
        let blocked_me = ProfileHelper::has_blocked(ctx, &orange_name, &my_orange_name);

        let bumper: Box<dyn Drawable> = is_blocked
            .then(|| format!("You blocked {}. Unblock to message.", username))
            .or_else(|| blocked_me.then(|| format!("{} has blocked you.", username)))
            .map(|msg| Box::new(Alert::new(ctx, msg.as_str())) as Box<dyn Drawable>)
            .unwrap_or_else(|| Box::new(TextInputMessages::new(ctx, *room_id)) as Box<dyn Drawable>);

        let offset = if room.messages.is_empty() {Offset::Center} else {Offset::End};

        let text_size = ctx.theme.fonts.size.md;
        let content = room.messages.is_empty().then(|| {
            let text = format!("No messages yet.\nSend {} the first message.", username);
            Box::new(Text::new(ctx, &text, TextStyle::Secondary, text_size, Align::Center)) as Box<dyn Drawable>
        }).unwrap_or_else(|| {
            Box::new(TextMessageGroup::new(ctx, &room.messages, MessageType::Contact)) as Box<dyn Drawable>
        });

        let account_return = Arc::new(Mutex::new(Some(account_return)));

        let back_account_return = account_return.clone();
        let back = IconButton::navigation(ctx, "left", move |ctx: &mut Context| {
            let page = back_account_return.lock().unwrap().take().unwrap();
            ctx.trigger_event(NavigateEvent(Some(page.0), page.1));
        });

        let info_orange_name = orange_name.clone();
        let info_account_return = account_return.clone();
        let room_id = room_id.clone();
        let info = IconButton::navigation(ctx, "info", move |ctx: &mut Context| {
            let (on_return, with_nav) = Self::new(ctx, &room_id, info_account_return.lock().unwrap().take().unwrap());
            let page = UserAccount::new(ctx, &info_orange_name, (on_return.into_boxed(), with_nav));
            ctx.trigger_event(NavigateEvent::new(page))
        });

        let bumper = Bumper::new(ctx, vec![bumper]);
        let content = Content::new(offset, vec![content]);
        let header = HeaderMessages::new(ctx, Some(back), Some(info), vec![orange_name]);
        (DirectMessage(Stack::center(), Page::new(header, content, Some(bumper)), room_id), false)
    }
}

impl OnEvent for DirectMessage {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let mut rooms = ctx.state().get::<Rooms>();
            let messages = &rooms.0.get_mut(&self.2).unwrap().messages;
            if !messages.is_empty() {
                if let Some(group) = &mut self.1.content().find::<TextMessageGroup>() {
                    if messages.len() > group.count() {
                        **group = TextMessageGroup::new(ctx, &messages, MessageType::Group);
                    }
                } else {
                    self.1.content().remove::<Text>();
                    let group = Box::new(TextMessageGroup::new(ctx, &messages, MessageType::Group)) as Box<dyn Drawable>;
                    self.1.content().items().push(group);
                    *self.1.content().offset() = Offset::End;
                }
            }
        }
        true
    }
}

#[derive(Debug, Component, AppPage)]
pub struct GroupMessage(Stack, Page, #[skip] uuid::Uuid);

impl GroupMessage {
    pub fn new(ctx: &mut Context, room_id: &uuid::Uuid) -> (Self, bool) {
        let rooms = ctx.state().get::<Rooms>();
        let room = rooms.0.get(room_id).unwrap();
        let room_id = room_id.clone();

        let offset = if room.messages.is_empty() {Offset::Center} else {Offset::End};
        let text_size = ctx.theme.fonts.size.md;
        let content = room.messages.is_empty().then(|| {
            Box::new(Text::new(ctx, "No messages yet.\nSend the first message.", TextStyle::Secondary, text_size, Align::Center)) as Box<dyn Drawable>
        }).unwrap_or_else(|| {
            Box::new(TextMessageGroup::new(ctx, &room.messages, MessageType::Group)) as Box<dyn Drawable>
        });

        let input = TextInputMessages::new(ctx, room_id);
       
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| {
            let page = MessagesHome::new(ctx);
            ctx.trigger_event(NavigateEvent::new(page))
        });

        let info = IconButton::navigation(ctx, "info", move |ctx: &mut Context| {
            let page = GroupInfo::new(ctx, &room_id);
            ctx.trigger_event(NavigateEvent::new(page))
        });

        let bumper = Bumper::new(ctx, vec![Box::new(input)]);
        let content = Content::new(offset, vec![content]);
        let header = HeaderMessages::new(ctx, Some(back), Some(info), room.authors.clone());
        (GroupMessage(Stack::center(), Page::new(header, content, Some(bumper)), room_id), false)
    }
}

impl OnEvent for GroupMessage {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let mut rooms = ctx.state().get::<Rooms>();
            let messages = &rooms.0.get_mut(&self.2).unwrap().messages;
            if !messages.is_empty() {
                if let Some(group) = &mut self.1.content().find::<TextMessageGroup>() {
                    if messages.len() > group.count() {
                        **group = TextMessageGroup::new(ctx, &messages, MessageType::Group);
                    }
                } else {
                    self.1.content().remove::<Text>();
                    let group = Box::new(TextMessageGroup::new(ctx, &messages, MessageType::Group)) as Box<dyn Drawable>;
                    self.1.content().items().push(group);
                    *self.1.content().offset() = Offset::End;
                }
            }
        }
        true
    }
}

#[derive(Debug, Component, AppPage)]
pub struct GroupInfo(Stack, Page);
impl OnEvent for GroupInfo {}

impl GroupInfo {
    pub fn new(ctx: &mut Context, room_id: &uuid::Uuid) -> (Self, bool) {
        let mut rooms = ctx.state().get::<Rooms>();
        let room = rooms.0.get_mut(&room_id).unwrap();
        let room_id = room_id.clone(); 
        let contacts = room.authors.iter().map(|orange_name| {
            let new_profile = orange_name.clone();
            ListItemMessages::contact(ctx, &orange_name,
                move |ctx: &mut Context| {
                    let (on_return, with_nav) = Self::new(ctx, &room_id);
                    let page = UserAccount::new(ctx, &new_profile.clone(), (on_return.into_boxed(), with_nav));
                    ctx.trigger_event(NavigateEvent::new(page))
                }
            )
        }).collect::<Vec<ListItem>>();

        let text_size = ctx.theme.fonts.size.md;
        let members = format!("This group has {} members.", contacts.len());
        let text = Text::new(ctx, &members, TextStyle::Secondary, text_size, Align::Center);
        let content = Content::new(Offset::Start, vec![Box::new(text), Box::new(ListItemGroup::new(contacts))]);

        let room_id = room_id.clone(); 
        let back = IconButton::navigation(ctx, "left", move |ctx: &mut Context| {
            let page = GroupMessage::new(ctx, &room_id);
            ctx.trigger_event(NavigateEvent::new(page))
        });

        let header = Header::stack(ctx, Some(back), "Group Message Info", None);
        (GroupInfo(Stack::center(), Page::new(header, content, None)), false)
    }
}

