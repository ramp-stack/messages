use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use profiles::service::Profiles;
use profiles::pages::{UserAccount, AccountActions};
use profiles::plugin::ProfilePlugin;
use pelican_ui::air::{OrangeName, Id};

use crate::components::{QuickDeselect, MessageType, ListItemMessages, ListItemGroupMessages, TextMessageGroup, TextInputMessages, HeaderMessages};
use crate::events::{CreateMessageEvent, OpenAccountEvent, SetRoomEvent};
use crate::plugin::MessagesPlugin;
use crate::service::{RoomsRequest, Rooms, Message};

use pelican_ui_std::{
    AppPage, Stack, Page,
    Header, IconButton, Text,
    ExpandableText, TextStyle, 
    Offset, ListItem, Content,
    Button, ButtonState, Searchbar,
    Bumper, TextInput, Alert,
    NavigateEvent, ListItemGroup,
};

use uuid::Uuid;

// use crate::MSGPlugin;
// use crate::msg::{CurrentRoom, CurrentProfile};

#[derive(Component)]
pub struct MessagesHome(Stack, Page, #[skip] Option<Id>, #[skip] Vec<(Id, Vec<OrangeName>, Vec<Message>)>, #[skip] AccountActions);

impl AppPage for MessagesHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(SelectRecipients::new(ctx, self.4))),
            1 => Ok(Box::new(GroupMessage::new(ctx, self.2.unwrap(), self.4))),
            2 => Ok(Box::new(DirectMessage::new(ctx, self.2.unwrap(), self.4, None))),
            _ => Err(self),
        }
    }
}

impl std::fmt::Debug for MessagesHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MessagesHome")
    }
}

impl MessagesHome {
    pub fn new(ctx: &mut Context, account_actions: AccountActions) -> Self {
        let header = Header::home(ctx, "Messages");
        let new_message = Button::primary(ctx, "New Message", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let bumper = Bumper::single_button(ctx, new_message);
        let rooms = ctx.state().get_or_default::<Rooms>().rooms();
        let text_size = ctx.theme.fonts.size.md;
        let instructions = ExpandableText::new(ctx, "No messages yet.\nGet started by messaging a friend.", TextStyle::Secondary, text_size, Align::Center, None);

        let content = match rooms.is_empty() {
            false => Content::new(Offset::Start, vec![Box::new(ListItemGroupMessages::new(ctx, rooms.clone()))]),
            true => Content::new(Offset::Center, vec![Box::new(instructions)])
        };

        MessagesHome(Stack::center(), Page::new(Some(header), content, Some(bumper)), None, rooms, account_actions)
    }
}

impl OnEvent for MessagesHome {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let rooms = ctx.state().get_or_default::<Rooms>().clone().rooms();
            if self.3 != rooms {
                if let Some(group) = self.1.content().find::<ListItemGroup>() {
                    *group = ListItemGroupMessages::new(ctx, rooms);
                } else {
                    self.3 = rooms.clone();
                    self.1.content().remove::<ExpandableText>();
                    let group = ListItemGroupMessages::new(ctx, rooms);
                    self.1.content().items().push(Box::new(group));
                    *self.1.content().offset() = Offset::Start;
                }
            }
        } else if let Some(SetRoomEvent(id)) = event.downcast_ref::<SetRoomEvent>() {
            self.2 = Some(*id);
        }
        true
    }
}

#[derive(Component)]
pub struct SelectRecipients(Stack, Page, #[skip] ButtonState, #[skip] Option<Id>, #[skip] AccountActions, #[skip] Option<Uuid>);

impl AppPage for SelectRecipients {
    fn has_nav(&self) -> bool { false }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(MessagesHome::new(ctx, self.4))),
            1 => Ok(Box::new(GroupMessage::new(ctx, self.3.unwrap(), self.4))),
            2 => Ok(Box::new(DirectMessage::new(ctx, self.3.unwrap(), self.4, None))),
            _ => Err(self),
        }
    }
}

impl std::fmt::Debug for SelectRecipients {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SelectRecipients")
    }
}

impl SelectRecipients {
    pub fn new(ctx: &mut Context, account_actions: AccountActions) -> Self {
        let icon_button = None::<(&'static str, fn(&mut Context, &mut String))>;
        let searchbar = Searchbar::new(TextInput::new(ctx, None, None, "Profile name...", None, icon_button));

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
        SelectRecipients(Stack::center(), Page::new(Some(header), content, Some(bumper)), ButtonState::Default, None, account_actions, None)
    }
}

impl OnEvent for SelectRecipients {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let error = self.1.content().find::<QuickDeselect>().map(|deselect| deselect.get_orange_names().is_none()).unwrap_or(true);

            let button = self.1.bumper().as_mut().unwrap().find::<Button>().unwrap();
            button.update_state(ctx, error, !error, &mut self.2);

            if let Some(uuid) = self.5 {
                println!("ID was some {:?}", uuid);
                let mut guard = ctx.get::<MessagesPlugin>();
                let (plugin, ctx) = guard.get();
                match ctx.state().get::<Rooms>().expect("no rooms in state").0.iter().find(|(u, _)| *u == uuid) {
                    Some((_, room)) => {
                        println!("Found room");
                        self.1.content().find::<QuickDeselect>().unwrap().get_orange_names().unwrap().iter().for_each(|on| {
                            plugin.request(RoomsRequest::Share(room.0, on.clone()));
                        });
                        
                        self.3 = Some(room.0);
                        let nav = if room.1.len() > 1 {1} else {2};
                        ctx.trigger_event(NavigateEvent(nav));
                    },
                    None => println!("Could not find room")
                }
            }
        } else if let Some(CreateMessageEvent) = event.downcast_ref::<CreateMessageEvent>() {
            let mut guard = ctx.get::<MessagesPlugin>();
            let plugin = guard.get().0;
            let uuid = uuid::Uuid::new_v4();
            println!("Creating room under id {:?}", uuid);
            plugin.request(RoomsRequest::CreateRoom(uuid));            
            self.5 = Some(uuid);

            // let orange_names = self.1.content().find::<QuickDeselect>().unwrap().get_orange_names().unwrap();
            // let mut rooms = ctx.state().get::<FakeRooms>();

            // for (id, room) in rooms.0.iter() {
            //     match room.authors.len() {
            //         1 if room.authors[0] == orange_names[0] => {
            //             // Direct already exists
            //             self.3 = Some(*id);
            //             ctx.trigger_event(NavigateEvent(2));
            //             return true;
            //         }
            //         _ => {
            //             let a: HashSet<_> = orange_names.iter().cloned().collect();
            //             let b: HashSet<_> = room.authors.iter().cloned().collect();

            //             if a == b {
            //                 // Group alerady exists
            //                 self.3 = Some(*id);
            //                 ctx.trigger_event(NavigateEvent(1));
            //                 return true;
            //             }
            //         }
            //     }
            // }

            // println!("create dm");
            // let id = uuid::Uuid::new_v4();
            // let trigger_event = match orange_names.len() > 1 { 
            //     true => |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)),
            //     false => |ctx: &mut Context| ctx.trigger_event(NavigateEvent(2))
            // };

            // self.3 = Some(id);
            // rooms.add(FakeRoom::new(orange_names.clone()), id);
            // ctx.state().set(&rooms);
            // trigger_event(ctx);
        }
        true
    }
}

#[derive(Component)]
pub struct DirectMessage(Stack, Page, #[skip] Id, #[skip] OrangeName, #[skip] Option<Box<dyn AppPage>>, #[skip] AccountActions);

impl AppPage for DirectMessage {
    fn has_nav(&self) -> bool { false }
    fn navigate(mut self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(self.4.take().unwrap_or(Box::new(MessagesHome::new(ctx, self.5)))),
            1 => Ok(Box::new(UserAccount::new(ctx, self.3.clone(), self.5.clone(), self))),
            _ => Err(self),
        }
    }
}

impl std::fmt::Debug for DirectMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DirectMessage")
    }
}

impl DirectMessage {
    pub fn new(ctx: &mut Context, room_id: Id, account_actions: AccountActions, account_return: Option<Box<dyn AppPage>>) -> Self {
        let room = ctx.state().get_mut_or_default::<Rooms>().get(room_id).unwrap().clone();
        let me = ProfilePlugin::me(ctx).0;
        let orange_name = room.1.into_iter().filter(|orange_name| *orange_name != me).collect::<Vec<_>>().first().unwrap_or(&me).clone();

        let username = ProfilePlugin::username(ctx, &orange_name); //ctx.state().get_or_default::<Profiles>().0.get(&orange_name).unwrap().clone();
        let is_blocked = ProfilePlugin::has_blocked(ctx, &me, &orange_name);
        let blocked_me = ProfilePlugin::has_blocked(ctx, &orange_name, &me);

        let bumper: Box<dyn Drawable> = is_blocked
            .then(|| format!("You blocked {}. Unblock to message.", username))
            .or_else(|| blocked_me.then(|| format!("{} has blocked you.", username)))
            .map(|msg| Box::new(Alert::new(ctx, msg.as_str())) as Box<dyn Drawable>)
            .unwrap_or_else(|| Box::new(TextInputMessages::new(ctx, room.0)) as Box<dyn Drawable>);

        let offset = if room.2.is_empty() {Offset::Center} else {Offset::End};
        let content = match room.2.is_empty() {
            true => {
                let text_size = ctx.theme.fonts.size.md;
                let text = format!("No messages yet.\nSend {} the first message.", username);
                Box::new(ExpandableText::new(ctx, &text, TextStyle::Secondary, text_size, Align::Center, None)) as Box<dyn Drawable>
            },
            false => Box::new(TextMessageGroup::new(ctx, &room.2, MessageType::Contact)) as Box<dyn Drawable>
        };

        let back = IconButton::navigation(ctx, "left", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let info = IconButton::navigation(ctx, "info", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));

        let bumper = Bumper::new(ctx, vec![bumper]);
        let content = Content::new(offset, vec![content]);
        let header = HeaderMessages::new(ctx, Some(back), Some(info), vec![orange_name.clone()]);
        DirectMessage(Stack::center(), Page::new(Some(header), content, Some(bumper)), room_id, orange_name, account_return, account_actions)
    }
}

impl OnEvent for DirectMessage {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let room = ctx.state().get_mut_or_default::<Rooms>().get(self.2).unwrap().clone();
            if !room.2.is_empty() {
                if let Some(group) = &mut self.1.content().find::<TextMessageGroup>() {
                    if room.2.len() > group.count() {
                        **group = TextMessageGroup::new(ctx, &room.2, MessageType::Group);
                    }
                } else {
                    self.1.content().remove::<ExpandableText>();
                    let group = Box::new(TextMessageGroup::new(ctx, &room.2, MessageType::Group)) as Box<dyn Drawable>;
                    self.1.content().items().push(group);
                    *self.1.content().offset() = Offset::End;
                }
            }
        }
        true
    }
}

#[derive(Component)]
pub struct GroupMessage(Stack, Page, #[skip] Id, #[skip] AccountActions);

impl AppPage for GroupMessage {
    fn has_nav(&self) -> bool { false }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(MessagesHome::new(ctx, self.3))),
            1 => Ok(Box::new(GroupInfo::new(ctx, self.2, self.3))),
            _ => Err(self),
        }
    }
}

impl std::fmt::Debug for GroupMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GroupMessage")
    }
}

impl GroupMessage {
    pub fn new(ctx: &mut Context, room_id: Id, account_actions: AccountActions) -> Self {
        let room = ctx.state().get_mut_or_default::<Rooms>().get(room_id).unwrap().clone();
        let offset = if room.2.is_empty() {Offset::Center} else {Offset::End};
        let text_size = ctx.theme.fonts.size.md;
        let content = match room.2.is_empty() {
            true => Box::new(ExpandableText::new(ctx, "No messages yet.\nSend the first message.", TextStyle::Secondary, text_size, Align::Center, None)) as Box<dyn Drawable>,
            false => Box::new(TextMessageGroup::new(ctx, &room.2, MessageType::Group)) as Box<dyn Drawable>
        };
        let input = TextInputMessages::new(ctx, room.0);
       
        let back = IconButton::navigation(ctx, "left", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let info = IconButton::navigation(ctx, "info", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));

        let bumper = Bumper::new(ctx, vec![Box::new(input)]);
        let content = Content::new(offset, vec![content]);
        let header = HeaderMessages::new(ctx, Some(back), Some(info), room.1.clone());
        GroupMessage(Stack::center(), Page::new(Some(header), content, Some(bumper)), room_id, account_actions)
    }
}

impl OnEvent for GroupMessage {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(TickEvent) = event.downcast_ref::<TickEvent>() {
            let room = ctx.state().get_mut_or_default::<Rooms>().get(self.2).unwrap().clone();
            if !room.2.is_empty() {
                if let Some(group) = &mut self.1.content().find::<TextMessageGroup>() {
                    if room.2.len() > group.count() {
                        **group = TextMessageGroup::new(ctx, &room.2, MessageType::Group);
                    }
                } else {
                    self.1.content().remove::<ExpandableText>();
                    let group = Box::new(TextMessageGroup::new(ctx, &room.2, MessageType::Group)) as Box<dyn Drawable>;
                    self.1.content().items().push(group);
                    *self.1.content().offset() = Offset::End;
                }
            }
        }
        true
    }
}

#[derive(Component)]
pub struct GroupInfo(Stack, Page, #[skip] Id, #[skip] Option<OrangeName>, #[skip] AccountActions);

impl AppPage for GroupInfo {
    fn has_nav(&self) -> bool { false }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        match index {
            0 => Ok(Box::new(GroupMessage::new(ctx, self.2, self.4))),
            1 => Ok(Box::new(UserAccount::new(ctx, self.3.as_ref().unwrap().clone(), self.4.clone(), self))),
            _ => Err(self),
        }
    }
}

impl std::fmt::Debug for GroupInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GroupInfo")
    }
}

impl GroupInfo {
    pub fn new(ctx: &mut Context, room_id: Id, account_actions: AccountActions) -> Self {
        let room = ctx.state().get_mut_or_default::<Rooms>().get(room_id).unwrap().clone();
        let contacts = room.1.iter().map(|orange_name| {
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
        GroupInfo(Stack::center(), Page::new(Some(header), content, None), room_id, None, account_actions)
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