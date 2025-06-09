use pelican_ui::events::OnEvent;
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use profiles::components::AvatarContentProfiles;
use profiles::service::{Profiles, Name};
use profiles::OrangeName;

use chrono::Duration;

use crate::{Message, Rooms};
use crate::components::AvatarMessages;

use pelican_ui_std::{
    TextInput, ClearActiveInput,
};

pub struct TextInputMessages;
impl TextInputMessages {
    pub fn new(ctx: &mut Context, current_room_id: uuid::Uuid) -> TextInput {
        TextInput::new(ctx, None, None, "Message...", None, 
            Some(("send", 
                move |ctx: &mut Context, string: &mut String| {
                    if !string.is_empty() {
                        let mut rooms = ctx.state().get::<Rooms>();
                        let room = rooms.0.get_mut(&current_room_id).unwrap();
                        let orange_name = ctx.state().get::<Name>().0.unwrap();
                        room.add_message(Message::from(string.to_string(), orange_name));
                        ctx.state().set(&rooms);
                        ctx.trigger_event(ClearActiveInput);
                    }
                }
            ))
        )
    }
}