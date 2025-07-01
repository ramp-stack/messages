use pelican_ui::Context;
use pelican_ui::air::Id;
use profiles::plugin::ProfilePlugin;
use crate::plugin::MessagesPlugin;
use crate::service::Message;
use pelican_ui_std::{TextInput, ClearActiveInput};

pub struct TextInputMessages;
impl TextInputMessages {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, current_room_id: Id) -> TextInput {
        TextInput::new(ctx, None, None, "Message...", None, 
            Some(("send", 
                move |ctx: &mut Context, string: &mut String| {
                    if !string.is_empty() {
                        let me = ProfilePlugin::me(ctx).0;
                        let message = Message::from(string.to_string(), me);
                        MessagesPlugin::create_message(ctx, current_room_id, message);
                        ctx.trigger_event(ClearActiveInput);
                    }
                }
            )),
            true,
        )
    }
}