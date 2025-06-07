use pelican_ui::Context;

use pelican_ui_std::{
    Avatar,
    AvatarContent,
    AvatarIconStyle,
};

pub struct AvatarMessages;
impl AvatarMessages {
    pub fn new(ctx: &mut Context, avatar_content: AvatarContent) -> Avatar {
        Avatar::new(ctx, avatar_content, None, false, 24.0, None)
    }
}
