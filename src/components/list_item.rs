#![allow(clippy::new_ret_no_self)]

use pelican_ui::events::{OnEvent, Event, MouseState, MouseEvent};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};
use profiles::plugin::ProfilePlugin;
use pelican_ui::air::OrangeName;

use profiles::components::AvatarContentProfiles;

use pelican_ui_std::{
    AdjustScrollEvent,
    SearchEvent,
    Stack, Button,
    Padding, Offset,
    Size, Wrap,
    ListItemGroup,
    Column, ListItem,
    AvatarContent,
    AvatarIconStyle,
    NavigateEvent,
};

use crate::events::{RemoveContactEvent, AddContactEvent, SetRoomEvent};
use crate::service::{Room, Message};

pub struct ListItemGroupMessages;

impl ListItemGroupMessages {
    pub fn new(ctx: &mut Context, mut rooms: Vec<Room>) -> ListItemGroup {
        rooms.sort_by_key(|room| room.2.last().map(|msg| *msg.timestamp()));

        let items = rooms.into_iter().rev().map(|room| {
            match room.1.len() > 2 {
                true => ListItemMessages::group_message(ctx, room.1.clone(), room.2.clone(), move |ctx: &mut Context| {
                    ctx.trigger_event(SetRoomEvent(room.0));
                    ctx.trigger_event(NavigateEvent(1));
                }),
                false => {
                    let me = ProfilePlugin::me(ctx).0;
                    let user = room.1.into_iter().filter(|orange_name| *orange_name != me).collect::<Vec<_>>().last().unwrap_or(&me).clone();
                    ListItemMessages::direct_message(ctx, user, room.2.clone(), move |ctx: &mut Context| {
                        ctx.trigger_event(SetRoomEvent(room.0));
                        ctx.trigger_event(NavigateEvent(2));
                    })
                }
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
        let orange = orange_name.to_string();
        let orange = orange.strip_prefix("orange_name:").unwrap_or(orange.as_str());
        ListItem::new(ctx, true, &name, None, Some(orange), None, None, None, None, Some(data), None, on_click)
    }

    pub fn recipient(ctx: &mut Context, orange_name: &OrangeName) -> ListItem {
        let name = ProfilePlugin::username(ctx, orange_name);
        let data = AvatarContentProfiles::from_orange_name(ctx, orange_name);
        let contact = orange_name.clone();
        let orange = contact.to_string();
        let orange = orange.strip_prefix("orange_name:").unwrap_or(orange.as_str());
        ListItem::new(
            ctx, true, &name, None, Some(orange), None, None, None, None, Some(data), None, 
            move |ctx: &mut Context| ctx.trigger_event(AddContactEvent(contact.clone()))
        )
    }

    pub fn direct_message(ctx: &mut Context, other: OrangeName, mut messages: Vec<Message>, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let me = ProfilePlugin::me(ctx).0;
        let other_name = ProfilePlugin::username(ctx, &other);
        let data = AvatarContentProfiles::from_orange_name(ctx, &other);
        messages.retain(|m| *m.message() != "__system__joined");
        let (recent, read) = &messages.last().map(|m| {
            let prefix = if *m.author() == me {"You".to_string()} else {other_name.clone()};
            (format!("{}: {}", prefix, m.message().clone()), *m.is_read())
        }).unwrap_or(("No messages yet.".to_string(), true));
        let color = ctx.theme.colors.brand.primary;
        ListItem::new(ctx, true, &other_name, (!read).then_some(("notification", color)), Some(recent), None, None, None, None, Some(data), None, on_click)
    }

    pub fn group_message(ctx: &mut Context, names: Vec<OrangeName>, messages: Vec<Message>, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let read = messages.last().map(|m| *m.is_read()).unwrap_or(true);
        let me = ProfilePlugin::me(ctx).0;
        let names = names.iter().filter(|orange| **orange != me).map(|orange_name| {
            ProfilePlugin::username(ctx, orange_name).trim().to_string()
        }).collect::<Vec<String>>();
        let names = names.join(", ");
        let avatar = AvatarContent::Icon("group", AvatarIconStyle::Secondary);
        let color = ctx.theme.colors.brand.primary;
        ListItem::new(ctx, true, "Group Message", (!read).then_some(("notification", color)), None, Some(&names), None, None, None, Some(avatar), None, on_click)
    }

    // pub fn room(ctx: &mut Context, data: AvatarContent, name: &str, members: &str, description: &str, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
    //     ListItem::new(ctx, true, name, None, Some(members), Some(description), None, None, None, Some(data), None, on_click)
    // }
}


#[derive(Debug, Component)]
pub struct QuickDeselect(Column, Option<QuickDeselectContent>, ListItemGroup, #[skip] f32);

impl QuickDeselect {
    pub fn new(list_items: Vec<ListItem>) -> Self {
        QuickDeselect(
            Column::new(24.0, Offset::Start, Size::Fit, Padding::default()), 
            None, 
            ListItemGroup::new(list_items),
            0.0
        )
    }

    pub fn get_orange_names(&self) -> Option<Vec<OrangeName>> {
        self.1.as_ref().map(|c| c.1.iter().map(|b| b.orange_name()).collect())
    }

    fn select_item(&mut self, ctx: &mut Context, orange_name: &OrangeName, select: bool) {
        self.2.items().iter_mut().filter_map(|l| {
            let comp = l.inner().subtitle().as_mut().map(|sb| sb.text().spans[0].text.clone()).unwrap_or_default();
            (Some(comp.as_str()) == orange_name.to_string().strip_prefix("orange_name:")).then(|| l.inner().title())
        }).for_each(|title| {
            // let new_flair = select.then(|| Flair::new(ctx, "checkmark", AvatarIconStyle::Success, avatar.as_mut().unwrap().size()));
            // *avatar.as_mut().unwrap().flair() = new_flair;
            let color = ctx.theme.colors.text.heading;
            let flair = select.then_some(("checkmark", color));
            title.update_flair(ctx, flair);
        });
    }
}

impl OnEvent for QuickDeselect {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(SearchEvent(query)) = event.downcast_ref::<SearchEvent>() {
            let query = query.to_lowercase();

            let mut items_and_flags: Vec<_> = self.2.items().drain(..).map(|mut item| {
                let name = item.inner().title().title().text().spans.first().map(|s| s.text.to_lowercase()).unwrap_or_default();
                let orange = item.inner().subtitle().as_mut().and_then(|sb| sb.text().spans.first().map(|s| s.text.to_lowercase())).unwrap_or_default();

                let priority = if query.is_empty() || name.contains(&query) {0} else if orange.contains(&query) {1} else {2};

                ((priority, 0), item, priority == 2)
            }).collect();

            items_and_flags.sort_by_key(|(key, _, _)| *key);
            let (items, flags): (Vec<_>, Vec<_>) = items_and_flags.into_iter().map(|(_, item, flag)| (item, flag)).unzip();

            *self.2.items() = items;
            flags.into_iter().enumerate().for_each(|(i, flag)| self.2.hide(flag, i));
        } else if let Some(AddContactEvent(orange_name)) = event.downcast_ref::<AddContactEvent>() {
            self.select_item(ctx, orange_name, true);

            let button = QuickDeselectButton::new(ctx, orange_name.clone());
            match &mut self.1 {
                Some(select) => {
                    if !select.1.iter().any(|selected| selected.orange_name() == *orange_name) {
                        select.1.push(button);
                        let adj = Drawable::request_size(self.1.as_ref().unwrap(), ctx).min_height();
                        ctx.trigger_event(AdjustScrollEvent(adj - self.3));
                        self.3 = adj;
                    } else {
                        ctx.trigger_event(RemoveContactEvent(orange_name.clone()));
                    }
                },
                None => {
                    self.1 = Some(QuickDeselectContent::new(button));
                    let adj = Drawable::request_size(self.1.as_ref().unwrap(), ctx).min_height();
                    ctx.trigger_event(AdjustScrollEvent((adj - self.3) + 24.0));
                    self.3 = adj;
                }
            }
        } else if let Some(RemoveContactEvent(orange_name)) = event.downcast_ref::<RemoveContactEvent>() {
            self.select_item(ctx, orange_name, false);

            if let Some(select) = &mut self.1 {
                if select.1.len() == 1 {
                    let adj = Drawable::request_size(self.1.as_ref().unwrap(), ctx).min_height();
                    self.1 = None;
                    ctx.trigger_event(AdjustScrollEvent(-(adj+24.0)));
                    self.3 = 0.0;
                } else {
                    select.1.retain(|button| button.orange_name() != *orange_name);
                    let adj = Drawable::request_size(self.1.as_ref().unwrap(), ctx).min_height();
                    ctx.trigger_event(AdjustScrollEvent(adj - self.3));
                    self.3 = adj;
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
        QuickDeselectContent(Wrap::new(8.0, 8.0), vec![first])
    }
}

#[derive(Debug, Component)]
struct QuickDeselectButton(Stack, Button, #[skip] OrangeName);

impl OnEvent for QuickDeselectButton {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(MouseEvent{state: MouseState::Released, position: Some(_)}) = event.downcast_ref::<MouseEvent>() {
            ctx.trigger_event(RemoveContactEvent(self.2.clone()))
        }
        true
    }
}

impl QuickDeselectButton {
    fn new(ctx: &mut Context, orange_name: OrangeName) -> Self {
        let name = ProfilePlugin::username(ctx, &orange_name);
        let button = Button::secondary(ctx, None, &name, Some("close"), move |_ctx: &mut Context| {}, None);
        QuickDeselectButton(Stack::default(), button, orange_name.clone())
    }

   fn orange_name(&self) -> OrangeName {self.2.clone()}
}