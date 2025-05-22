use rust_on_rails::prelude::*;
use pelican_ui::prelude::*;
use pelican_ui_profiles::Profile;

use crate::events::{RemoveContactEvent, AddContactEvent};

pub trait ListItemMessages {
    fn contact(ctx: &mut Context, data: AvatarContent, name: &str, nym: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self;
    fn recipient(ctx: &mut Context, data: AvatarContent, profile: Profile) -> Self;
    fn direct_message(ctx: &mut Context, data: AvatarContent, name: &str, recent: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self;
    fn group_message(ctx: &mut Context, names: Vec<&str>, on_click: impl FnMut(&mut Context) + 'static) -> Self;
    fn room(ctx: &mut Context, data: AvatarContent, name: &str, members: &str, description: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self;
}

impl ListItemMessages for ListItem {
    /// Creates a list item for a group text message member.
    fn contact(ctx: &mut Context, data: AvatarContent, name: &str, nym: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        ListItem::new(ctx, true, name, None, Some(nym), None, None, None, None, Some(data), None, on_click)
    }

    /// Creates a list item for a text message recipient selector.
    /// This method also triggers the `AddContactEvent` when clicked.
    fn recipient(ctx: &mut Context, data: AvatarContent, profile: Profile) -> Self {
        let p = profile.clone();
        ListItem::new(
            ctx, true, &profile.user_name, None, Some(&profile.identifier), None, None, None, None, Some(data), None, 
            move |ctx: &mut Context| ctx.trigger_event(AddContactEvent(p.clone()))
        )
    }

    /// Creates a list item for a direct message.
    /// Displays the most recent message along with the avatar and user details.
    fn direct_message(ctx: &mut Context, data: AvatarContent, name: &str, recent: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        ListItem::new(ctx, true, name, None, Some(recent), None, None, None, None, Some(data), None, on_click)
    }

    /// Creates a list item for a group message.
    /// Displays the names of the group members as the description.
    fn group_message(ctx: &mut Context, names: Vec<&str>, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        let description = names.join(", ");
        let avatar = AvatarContent::Icon("group", AvatarIconStyle::Secondary);
        ListItem::new(ctx, true, "Group Message", None, None, Some(&description), None, None, None, Some(avatar), None, on_click)
    }

    /// Creates a list item for a public room.
    /// Displays room details, including member count and description.
    fn room(ctx: &mut Context, data: AvatarContent, name: &str, members: &str, description: &str, on_click: impl FnMut(&mut Context) + 'static) -> Self {
        ListItem::new(ctx, true, name, None, Some(members), Some(description), None, None, None, Some(data), None, on_click)
    }
}


/// A component for quickly deselecting items (contacts) in a list.
#[derive(Debug, Component)]
pub struct QuickDeselect(Column, Option<QuickDeselectContent>, ListItemGroup);

impl QuickDeselect {
    /// Creates a new `QuickDeselect` component with a group of selectable list items.
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