#![allow(clippy::new_ret_no_self)]

use pelican_ui::events::{OnEvent, Event};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use profiles::plugin::ProfilePlugin;
use pelican_ui::air::OrangeName;

use profiles::components::AvatarContentProfiles;

use pelican_ui_std::{
    Stack, Button,
    Padding, Offset,
    Size, Wrap,
    ListItemGroup,
    Column,
    ListItem,
    AvatarContent,
    AvatarIconStyle,
    NavigateEvent,
};

use crate::events::{RemoveContactEvent, AddContactEvent, SetRoomEvent};
use crate::service::{Room, Message};

pub struct ListItemGroupMessages;

impl ListItemGroupMessages {
    pub fn new(ctx: &mut Context, rooms: Vec<Room>) -> ListItemGroup {
        let items = rooms.into_iter().map(|room| {
            match room.1.len() > 1 {
                true => ListItemMessages::group_message(ctx, room.1.clone(), move |ctx: &mut Context| {
                    ctx.trigger_event(SetRoomEvent(room.0));
                    ctx.trigger_event(NavigateEvent(1));
                }),
                false => ListItemMessages::direct_message(ctx, room.1.clone(), room.2.clone(), move |ctx: &mut Context| {
                    ctx.trigger_event(SetRoomEvent(room.0));
                    ctx.trigger_event(NavigateEvent(2));
                })
            }
        }).collect::<Vec<ListItem>>();
        ListItemGroup::new(items)
    }
}

pub struct ListItemMessages;

impl ListItemMessages {
    pub fn contact(ctx: &mut Context, orange_name: &OrangeName, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let name = ProfilePlugin::username(ctx, orange_name);
        let data = AvatarContentProfiles::from_orange_name(ctx, orange_name);
        ListItem::new(ctx, true, &name, None, Some(orange_name.to_string().as_str()), None, None, None, None, Some(data), None, on_click)
    }

    pub fn recipient(ctx: &mut Context, orange_name: &OrangeName) -> ListItem {
        let name = ProfilePlugin::username(ctx, orange_name);
        let data = AvatarContentProfiles::from_orange_name(ctx, orange_name);
        let contact = orange_name.clone();
        ListItem::new(
            ctx, true, &name, None, Some(contact.to_string().as_str()), None, None, None, None, Some(data), None, 
            move |ctx: &mut Context| ctx.trigger_event(AddContactEvent(contact.clone()))
        )
    }

    pub fn direct_message(ctx: &mut Context, names: Vec<OrangeName>, messages: Vec<Message>, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let (prefix, name, orange_name) = match names.first() {
            Some(o_n) => {
                let name = ProfilePlugin::username(ctx, o_n);
                (name.clone(), name, o_n.clone())
            }
            None => {
                let me = ProfilePlugin::me(ctx).0;
                ("You".to_string(), ProfilePlugin::username(ctx, &me), me)
            }
        };
        
        let data = AvatarContentProfiles::from_orange_name(ctx, &orange_name);
        let recent = &messages.last().map(|m| format!("{}: {}", prefix, m.message().clone())).unwrap_or("No messages yet.".to_string());
        ListItem::new(ctx, true, &name, None, Some(recent), None, None, None, None, Some(data), None, on_click)
    }

    pub fn group_message(ctx: &mut Context, names: Vec<OrangeName>, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        // let rooms = ctx.state().get::<FakeRooms>();
        // let room = rooms.0.get(room_id).unwrap();
        let names = names.iter().map(|orange_name| {
            ProfilePlugin::username(ctx, orange_name)
        }).collect::<Vec<String>>();
        let names = names.join(", ");
        let avatar = AvatarContent::Icon("group", AvatarIconStyle::Secondary);
        ListItem::new(ctx, true, "Group Message", None, None, Some(&names), None, None, None, Some(avatar), None, on_click)
    }

    // pub fn room(ctx: &mut Context, data: AvatarContent, name: &str, members: &str, description: &str, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
    //     ListItem::new(ctx, true, name, None, Some(members), Some(description), None, None, None, Some(data), None, on_click)
    // }
}


#[derive(Debug, Component)]
pub struct QuickDeselect(Column, Option<QuickDeselectContent>, ListItemGroup);

impl QuickDeselect {
    pub fn new(list_items: Vec<ListItem>) -> Self {
        QuickDeselect(
            Column::new(24.0, Offset::Start, Size::Fit, Padding::default()), 
            None, 
            ListItemGroup::new(list_items)
        )
    }

    pub fn get_orange_names(&self) -> Option<Vec<OrangeName>> {
        self.1.as_ref().map(|c| c.1.iter().map(|b| b.orange_name()).collect())
    }
}

impl OnEvent for QuickDeselect {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(AddContactEvent(orange_name)) = event.downcast_ref::<AddContactEvent>() {
            let button = QuickDeselectButton::new(ctx, orange_name.clone());
            match &mut self.1 {
                Some(select) => {
                    if !select.1.iter().any(|selected| selected.orange_name() == *orange_name) {select.1.push(button)}
                },
                None => self.1 = Some(QuickDeselectContent::new(button)),
            }
        } else if let Some(RemoveContactEvent(orange_name)) = event.downcast_ref::<RemoveContactEvent>() {
            if let Some(select) = &mut self.1 {
                if select.1.len() == 1 {
                    self.1 = None;
                } else {
                    select.1.retain(|button| button.orange_name() != *orange_name);
                }
            }
        }
        true
    }
}

#[derive(Debug, Component)]
struct QuickDeselectContent(Wrap, Vec<QuickDeselectButton>);
impl OnEvent for QuickDeselectContent {}

impl QuickDeselectContent {
    fn new(first: QuickDeselectButton) -> Self {
        QuickDeselectContent(
            Wrap(8.0, 8.0, Offset::Start, Offset::Center, Padding::default()), 
            vec![first],
        )
    }
}

#[derive(Debug, Component)]
struct QuickDeselectButton(Stack, Button, #[skip] OrangeName);
impl OnEvent for QuickDeselectButton {}

impl QuickDeselectButton {
    fn new(ctx: &mut Context, orange_name: OrangeName) -> Self {
        let name = ProfilePlugin::username(ctx, &orange_name);
        let contact_name = orange_name.clone();
        let button = Button::secondary(ctx, None, &name, Some("close"), move |ctx: &mut Context| {
            ctx.trigger_event(RemoveContactEvent(contact_name.clone()))
        });
        QuickDeselectButton(Stack::default(), button, orange_name.clone())
    }

   fn orange_name(&self) -> OrangeName {self.2.clone()}
}