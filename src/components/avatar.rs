use pelican_ui::events::OnEvent;
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

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
