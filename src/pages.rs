use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use profiles::service::Profiles;
use profiles::pages::UserAccount;
use profiles::plugin::ProfilePlugin;
use profiles::OrangeName;

use crate::{Room, Rooms};
use crate::components::{QuickDeselect, MessageType, ListItemMessages, TextMessageGroup, TextInputMessages, HeaderMessages};
use crate::events::{CreateMessageEvent, OpenAccountEvent, SetRoomIdEvent};

use pelican_ui_std::{
    AppPage, Stack, Page,
    Header, IconButton, Text,
    ExpandableText, TextStyle, 
    Offset, ListItem, Content,
    Button, ButtonState,
    Bumper, TextInput, Alert,
    NavigateEvent, ListItemGroup,
};

use std::collections::HashSet;

// use crate::MSGPlugin;
// use crate::msg::{CurrentRoom, CurrentProfile};

#[derive(Debug, Component)]
pub struct MessagesHome(Stack, Page, #[skip] Option<uuid::Uuid>);

impl AppPage for MessagesHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(SelectRecipients::new(ctx))),
            1 => Ok(Box::new(GroupMessage::new(ctx, self.2.unwrap()))),
            2 => Ok(Box::new(DirectMessage::new(ctx, self.2.unwrap(), self))),
            _ => Err(self),
        }
    }
}

impl MessagesHome {
    pub fn new(ctx: &mut Context) -> Self {
        let header = Header::home(ctx, "Messages");
        let new_message = Button::primary(ctx, "New Message", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let bumper = Bumper::single_button(ctx, new_message);
        let rooms = ctx.state().get::<Rooms>().0;
        // println!("rooms {:?}", rooms);
        let messages = rooms.into_iter().map(|(id, room)| {
            match room.authors.len() > 1 {
                true => ListItemMessages::group_message(ctx, &id, move |ctx: &mut Context| {
                    ctx.trigger_event(SetRoomIdEvent(id));
                    ctx.trigger_event(NavigateEvent(1));
                }),
                false => ListItemMessages::direct_message(ctx, &id, move |ctx: &mut Context| {
                    ctx.trigger_event(SetRoomIdEvent(id));
                    ctx.trigger_event(NavigateEvent(2));
                })
            }
        }).collect::<Vec<ListItem>>();
        let text_size = ctx.theme.fonts.size.md;
        let instructions = ExpandableText::new(ctx, "No messages yet.\nGet started by messaging a friend.", TextStyle::Secondary, text_size, Align::Center);

        let content = match !messages.is_empty() {
            true => Content::new(Offset::Start, vec![Box::new(ListItemGroup::new(messages))]),
            false => Content::new(Offset::Center, vec![Box::new(instructions)])
        };

        MessagesHome(Stack::center(), Page::new(header, content, Some(bumper)), None)
    }
}

impl OnEvent for MessagesHome {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(SetRoomIdEvent(id)) = event.downcast_ref::<SetRoomIdEvent>() {
            self.2 = Some(*id);
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct SelectRecipients(Stack, Page, #[skip] ButtonState, #[skip] Option<uuid::Uuid>);

impl AppPage for SelectRecipients {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(MessagesHome::new(ctx))),
            1 => Ok(Box::new(GroupMessage::new(ctx, self.3.unwrap()))),
            2 => {
                let home = MessagesHome::new(ctx);
                Ok(Box::new(DirectMessage::new(ctx, self.3.unwrap(), Box::new(home))))
            },
            _ => Err(self),
        }
    }
}

impl SelectRecipients {
    pub fn new(ctx: &mut Context) -> Self {
        let icon_button = None::<(&'static str, fn(&mut Context, &mut String))>;
        let searchbar = TextInput::new(ctx, None, None, "Profile name...", None, icon_button);

        let profiles = ctx.state().get::<Profiles>().0;

        let recipients = profiles.keys().map(|orange_name| {
            ListItemMessages::recipient(ctx, orange_name)
        }).collect::<Vec<ListItem>>();
        
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
        SelectRecipients(Stack::center(), Page::new(header, content, Some(bumper)), ButtonState::Default, None)
    }
}

impl OnEvent for SelectRecipients {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let error = self.1.content().find::<QuickDeselect>().unwrap().get_orange_names().is_none();
            let button = self.1.bumper().as_mut().unwrap().find::<Button>().unwrap();
            button.update_state(ctx, error, !error, &mut self.2);
        } else if let Some(CreateMessageEvent) = event.downcast_ref::<CreateMessageEvent>() {
            let orange_names = self.1.content().find::<QuickDeselect>().unwrap().get_orange_names().unwrap();
            let mut rooms = ctx.state().get::<Rooms>();

            for (id, room) in rooms.0.iter() {
                match room.authors.len() {
                    1 if room.authors[0] == orange_names[0] => {
                        // Direct already exists
                        self.3 = Some(*id);
                        ctx.trigger_event(NavigateEvent(2));
                        return true;
                    }
                    _ => {
                        let a: HashSet<_> = orange_names.iter().cloned().collect();
                        let b: HashSet<_> = room.authors.iter().cloned().collect();

                        if a == b {
                            // Group alerady exists
                            self.3 = Some(*id);
                            ctx.trigger_event(NavigateEvent(1));
                            return true;
                        }
                    }
                }
            }

            println!("create dm");
            let id = uuid::Uuid::new_v4();
            let trigger_event = match orange_names.len() > 1 { 
                true => |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)),
                false => |ctx: &mut Context| ctx.trigger_event(NavigateEvent(2))
            };

            self.3 = Some(id);
            rooms.add(Room::new(orange_names.clone()), id);
            ctx.state().set(&rooms);
            trigger_event(ctx);
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct DirectMessage(Stack, Page, #[skip] uuid::Uuid, #[skip] OrangeName, #[skip] Option<Box<dyn AppPage>>);

impl AppPage for DirectMessage {
    fn has_nav(&self) -> bool { true }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(self.4.take().unwrap()),
            1 => Ok(Box::new(UserAccount::new(ctx, self.3, self.4.take().unwrap()))),
            _ => Err(self),
        }
    }
}

impl DirectMessage {
    pub fn new(ctx: &mut Context, room_id: uuid::Uuid, account_return: Box<dyn AppPage>) -> Self {
        let rooms = ctx.state().get::<Rooms>();
        let room = rooms.0.get(&room_id).unwrap();

        let profiles = ctx.state().get::<Profiles>();
        let orange_name = room.authors[0].clone();

        let user = profiles.0.get(&orange_name).unwrap();
        let username = user.get("username").unwrap();

        let my_orange_name = ProfilePlugin::me(ctx).unwrap().0;
        let is_blocked = ProfilePlugin::has_blocked(ctx, &my_orange_name, &orange_name);
        let blocked_me = ProfilePlugin::has_blocked(ctx, &orange_name, &my_orange_name);

        let bumper: Box<dyn Drawable> = is_blocked
            .then(|| format!("You blocked {}. Unblock to message.", username))
            .or_else(|| blocked_me.then(|| format!("{} has blocked you.", username)))
            .map(|msg| Box::new(Alert::new(ctx, msg.as_str())) as Box<dyn Drawable>)
            .unwrap_or_else(|| Box::new(TextInputMessages::new(ctx, room_id)) as Box<dyn Drawable>);

        let offset = if room.messages.is_empty() {Offset::Center} else {Offset::End};

        let text_size = ctx.theme.fonts.size.md;
        let content = match room.messages.is_empty() {
            true => {
                let text = format!("No messages yet.\nSend {} the first message.", username);
                Box::new(Text::new(ctx, &text, TextStyle::Secondary, text_size, Align::Center)) as Box<dyn Drawable>
            },
            false => Box::new(TextMessageGroup::new(ctx, &room.messages, MessageType::Contact)) as Box<dyn Drawable>
        };

        let back = IconButton::navigation(ctx, "left", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let info = IconButton::navigation(ctx, "info", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));

        let bumper = Bumper::new(ctx, vec![bumper]);
        let content = Content::new(offset, vec![content]);
        let header = HeaderMessages::new(ctx, Some(back), Some(info), vec![orange_name.clone()]);
        DirectMessage(Stack::center(), Page::new(header, content, Some(bumper)), room_id, orange_name, Some(account_return))
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
                        **group = TextMessageGroup::new(ctx, messages, MessageType::Group);
                    }
                } else {
                    self.1.content().remove::<Text>();
                    let group = Box::new(TextMessageGroup::new(ctx, messages, MessageType::Group)) as Box<dyn Drawable>;
                    self.1.content().items().push(group);
                    *self.1.content().offset() = Offset::End;
                }
            }
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct GroupMessage(Stack, Page, #[skip] uuid::Uuid);

impl AppPage for GroupMessage {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(MessagesHome::new(ctx))),
            1 => Ok(Box::new(GroupInfo::new(ctx, self.2))),
            _ => Err(self),
        }
    }
}

impl GroupMessage {
    pub fn new(ctx: &mut Context, room_id: uuid::Uuid) -> Self {
        let rooms = ctx.state().get::<Rooms>();
        let room = rooms.0.get(&room_id).unwrap();

        let offset = if room.messages.is_empty() {Offset::Center} else {Offset::End};
        let text_size = ctx.theme.fonts.size.md;
        let content = match room.messages.is_empty() {
            true => Box::new(Text::new(ctx, "No messages yet.\nSend the first message.", TextStyle::Secondary, text_size, Align::Center)) as Box<dyn Drawable>,
            false => Box::new(TextMessageGroup::new(ctx, &room.messages, MessageType::Group)) as Box<dyn Drawable>
        };
        let input = TextInputMessages::new(ctx, room_id);
       
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let info = IconButton::navigation(ctx, "info", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));

        let bumper = Bumper::new(ctx, vec![Box::new(input)]);
        let content = Content::new(offset, vec![content]);
        let header = HeaderMessages::new(ctx, Some(back), Some(info), room.authors.clone());
        GroupMessage(Stack::center(), Page::new(header, content, Some(bumper)), room_id)
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
                        **group = TextMessageGroup::new(ctx, messages, MessageType::Group);
                    }
                } else {
                    self.1.content().remove::<Text>();
                    let group = Box::new(TextMessageGroup::new(ctx, messages, MessageType::Group)) as Box<dyn Drawable>;
                    self.1.content().items().push(group);
                    *self.1.content().offset() = Offset::End;
                }
            }
        }
        true
    }
}

#[derive(Debug, Component)]
pub struct GroupInfo(Stack, Page, #[skip] uuid::Uuid, #[skip] Option<OrangeName>);

impl AppPage for GroupInfo {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(GroupMessage::new(ctx, self.2))),
            1 => Ok(Box::new(UserAccount::new(ctx, self.3.as_ref().unwrap().clone(), self))),
            _ => Err(self),
        }
    }
}

impl GroupInfo {
    pub fn new(ctx: &mut Context, room_id: uuid::Uuid) -> Self {
        let mut rooms = ctx.state().get::<Rooms>();
        let room = rooms.0.get_mut(&room_id).unwrap();
        let contacts = room.authors.iter().map(|orange_name| {
            let new_profile = orange_name.clone();
            ListItemMessages::contact(ctx, orange_name, move |ctx: &mut Context| {
                ctx.trigger_event(OpenAccountEvent(new_profile.clone()));
                ctx.trigger_event(NavigateEvent(1));
            })
        }).collect::<Vec<ListItem>>();

        let text_size = ctx.theme.fonts.size.md;
        let members = format!("This group has {} members.", contacts.len());
        let text = Text::new(ctx, &members, TextStyle::Secondary, text_size, Align::Center);
        let content = Content::new(Offset::Start, vec![Box::new(text), Box::new(ListItemGroup::new(contacts))]);
 
        let back = IconButton::navigation(ctx, "left", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let header = Header::stack(ctx, Some(back), "Group Message Info", None);
        GroupInfo(Stack::center(), Page::new(header, content, None), room_id, None)
    }
}

impl OnEvent for GroupInfo {
    fn on_event(&mut self, _ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(OpenAccountEvent(orange_name)) = event.downcast_ref::<OpenAccountEvent>() {
            self.3 = Some(orange_name.clone());
        }
        true
    }
}