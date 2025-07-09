use pelican_ui::events::{Event, OnEvent, TickEvent};
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component, resources};
use profiles::service::Profiles;
use profiles::pages::{UserAccount, AccountActions};
use profiles::plugin::ProfilePlugin;
use pelican_ui::air::{OrangeName, Id};

use crate::components::{Cards, QuickDeselect, MessageType, ListItemMessages, ListItemGroupMessages, TextMessageGroup, TextInputMessages, HeaderMessages};
use crate::events::{CreateMessageEvent, OpenAccountEvent, SetRoomEvent};
use crate::plugin::MessagesPlugin;
use crate::service::{RoomsRequest, Rooms, Message, PublicRoom, PublicRooms};
use crate::pages::{DirectMessage, GroupMessage, SelectRecipients};

use pelican_ui_std::{
    AppPage, Stack, Page,
    Header, IconButton, Text,
    ExpandableText, TextStyle, 
    Offset, ListItem, Content,
    Button, ButtonState, Searchbar,
    Bumper, TextInput, Alert,
    NavigateEvent, ListItemGroup,
    AvatarContent
};

use uuid::Uuid;

// use crate::MSGPlugin;
// use crate::msg::{CurrentRoom, CurrentProfile};

#[derive(Component)]
pub struct RoomsHome(Stack, Page, #[skip] Option<Id>, #[skip] Vec<(Id, Vec<OrangeName>, Vec<Message>)>, #[skip] AccountActions);

impl AppPage for RoomsHome {
    fn has_nav(&self) -> bool { true }
    fn navigate(self: Box<Self>, ctx: &mut Context, index: usize) -> Result<Box<dyn AppPage>, Box<dyn AppPage>> { 
        Ok(self)
    }
}

impl std::fmt::Debug for RoomsHome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RoomsHome")
    }
}

impl RoomsHome {
    pub fn new(ctx: &mut Context, account_actions: AccountActions) -> Self {
        let header = Header::home(ctx, "Rooms");
        let new_message = Button::primary(ctx, "Create Room", |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));

        let bumper = Bumper::single_button(ctx, new_message);
        // let rooms = ctx.state().get_or_default::<Rooms>().rooms();
        let rooms = PublicRooms::default();
        let cards = Cards::from(ctx, rooms.inner());

        let content = Content::new(Offset::Start, vec![Box::new(cards)]);
        RoomsHome(Stack::center(), Page::new(Some(header), content, Some(bumper)), None, Vec::new(), account_actions)
    }
}

impl OnEvent for RoomsHome {
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

