use pelican_ui::Context;
use profiles::plugin::Name;

use crate::{Message, Rooms};

use pelican_ui_std::{TextInput, ClearActiveInput};

pub struct TextInputMessages;
impl TextInputMessages {
    #[allow(clippy::new_ret_no_self)]
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