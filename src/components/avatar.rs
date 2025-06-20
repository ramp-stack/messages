use pelican_ui::Context;

use pelican_ui_std::{Avatar, AvatarContent};

pub struct AvatarMessages;
impl AvatarMessages {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, avatar_content: AvatarContent) -> Avatar {
        Avatar::new(ctx, avatar_content, None, false, 24.0, None)
    }
}
