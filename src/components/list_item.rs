use pelican_ui::events::{OnEvent, Event};
use pelican_ui::drawable::{Drawable, Component};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use profiles::Profile;

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

pub struct ListItemMessages;

impl ListItemMessages {
    pub fn contact(ctx: &mut Context, data: AvatarContent, name: &str, nym: &str, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        ListItem::new(ctx, true, name, None, Some(nym), None, None, None, None, Some(data), None, on_click)
    }

    pub fn recipient(ctx: &mut Context, data: AvatarContent, profile: Profile) -> ListItem {
        let p = profile.clone();
        ListItem::new(
            ctx, true, &profile.user_name, None, Some(&profile.identifier), None, None, None, None, Some(data), None, 
            move |ctx: &mut Context| ctx.trigger_event(AddContactEvent(p.clone()))
        )
    }

    pub fn direct_message(ctx: &mut Context, data: AvatarContent, name: &str, recent: &str, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        ListItem::new(ctx, true, name, None, Some(recent), None, None, None, None, Some(data), None, on_click)
    }

    pub fn group_message(ctx: &mut Context, names: Vec<String>, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        let description = names.join(", ");
        let avatar = AvatarContent::Icon("group", AvatarIconStyle::Secondary);
        ListItem::new(ctx, true, "Group Message", None, None, Some(&description), None, None, None, Some(avatar), None, on_click)
    }

    pub fn room(ctx: &mut Context, data: AvatarContent, name: &str, members: &str, description: &str, on_click: impl FnMut(&mut Context) + 'static) -> ListItem {
        ListItem::new(ctx, true, name, None, Some(members), Some(description), None, None, None, Some(data), None, on_click)
    }
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

    pub fn get_profiles(&self) -> Option<Vec<Profile>> {
        self.1.as_ref().map(|c| c.1.iter().map(|b| b.profile()).collect())
    }
}

impl OnEvent for QuickDeselect {
    fn on_event(&mut self, ctx: &mut Context, event: &mut dyn Event) -> bool {
        if let Some(AddContactEvent(profile)) = event.downcast_ref::<AddContactEvent>() {
            let button = QuickDeselectButton::new(ctx, profile.clone());
            match &mut self.1 {
                Some(select) => {
                    if !select.1.iter().any(|b| b.profile() == *profile) {select.1.push(button)}
                },
                None => self.1 = Some(QuickDeselectContent::new(button)),
            }
        } else if let Some(RemoveContactEvent(profile)) = event.downcast_ref::<RemoveContactEvent>() {
            if let Some(select) = &mut self.1 {
                if select.1.len() == 1 {
                    self.1 = None;
                } else {
                    select.1.retain(|button| button.profile() != *profile);
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
struct QuickDeselectButton(Stack, Button, #[skip] Profile);
impl OnEvent for QuickDeselectButton {}

impl QuickDeselectButton {
    fn new(ctx: &mut Context, profile: Profile) -> Self {
        let p = profile.clone();
        let button = Button::secondary(ctx, None, &p.user_name, Some("close"), move |ctx: &mut Context| {
            ctx.trigger_event(RemoveContactEvent(profile.clone()))
        });
        QuickDeselectButton(Stack::default(), button, p)
    }

   fn profile(&self) -> Profile {self.2.clone()}
}