use pelican_ui::events::{OnEvent, Event};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use profiles::service::Profiles;
use maverick_os::air::air;
use air::orange_name::OrangeName;

use profiles::components::AvatarContentProfiles;

use pelican_ui_std::{
    Stack,
    Button,
    Padding,
    Offset,
    Size,
    Wrap,
    ListItemGroup,
    Column,
    ListItem,
    AvatarContent,
    AvatarIconStyle
};

use crate::events::{RemoveContactEvent, AddContactEvent};
use crate::Rooms;

pub struct ListItemMessages;

impl ListItemMessages {
    pub fn contact(ctx: &mut Context, orange_name: &OrangeName, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let profiles = ctx.state().get::<Profiles>();
        let profile = profiles.0.get(orange_name).unwrap();
        let name = profile.get("username").unwrap();
        let data = AvatarContentProfiles::from_orange_name(ctx, orange_name);
        ListItem::new(ctx, true, name, None, Some(orange_name.to_string().as_str()), None, None, None, None, Some(data), None, on_click)
    }

    pub fn recipient(ctx: &mut Context, orange_name: &OrangeName) -> ListItem {
        let profiles = ctx.state().get::<Profiles>();
        let profile = profiles.0.get(orange_name).unwrap();
        let name = profile.get("username").unwrap();
        let data = AvatarContentProfiles::from_orange_name(ctx, orange_name);
        let contact = orange_name.clone();
        ListItem::new(
            ctx, true, name, None, Some(contact.to_string().as_str()), None, None, None, None, Some(data), None, 
            move |ctx: &mut Context| ctx.trigger_event(AddContactEvent(contact.clone()))
        )
    }

    pub fn direct_message(ctx: &mut Context, room_id: &uuid::Uuid, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let rooms = ctx.state().get::<Rooms>();
        let room = rooms.0.get(room_id).unwrap();
        let orange_name = &room.authors[0];
        let profiles = ctx.state().get::<Profiles>();
        let profile = profiles.0.get(orange_name).unwrap();
        let name = profile.get("username").unwrap();
        let data = AvatarContentProfiles::from_orange_name(ctx, orange_name);
        let recent = &room.messages.last().map(|m| m.message.clone()).unwrap_or("No messages yet.".to_string());
        ListItem::new(ctx, true, name, None, Some(recent), None, None, None, None, Some(data), None, on_click)
    }

    pub fn group_message(ctx: &mut Context, room_id: &uuid::Uuid, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let rooms = ctx.state().get::<Rooms>();
        let room = rooms.0.get(room_id).unwrap();
        let names = room.authors.iter().map(|orange_name| {
            let profiles = ctx.state().get::<Profiles>();
            let profile = profiles.0.get(orange_name).unwrap();
            profile.get("username").unwrap().to_string()
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
        let profiles = ctx.state().get::<Profiles>();
        let profile = profiles.0.get(&orange_name).unwrap();
        let name = profile.get("username").unwrap();
        let contact_name = orange_name.clone();
        let button = Button::secondary(ctx, None, name, Some("close"), move |ctx: &mut Context| {
            ctx.trigger_event(RemoveContactEvent(contact_name.clone()))
        });
        QuickDeselectButton(Stack::default(), button, orange_name.clone())
    }

   fn orange_name(&self) -> OrangeName {self.2.clone()}
}