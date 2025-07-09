use pelican_ui::Context;
use pelican_ui::resources;

use pelican_ui_std::{Avatar, AvatarContent, AvatarIconStyle};

pub struct AvatarMessages;
impl AvatarMessages {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, avatar_content: AvatarContent) -> Avatar {
        Avatar::new(ctx, avatar_content, None, false, 24.0, None)
    }

    pub fn room(ctx: &mut Context, content: AvatarContent) -> Avatar {
        Avatar::new(ctx, content, None, false, 128.0, None)
    }
}

pub struct AvatarContentMessages;

impl AvatarContentMessages {
    pub fn room(image: Option<resources::Image>) -> AvatarContent {
        match image {
            None => AvatarContentMessages::default(),
            Some(i) => AvatarContent::Image(i)
        }
    }

    pub fn default() -> AvatarContent {
        AvatarContent::Icon("door", AvatarIconStyle::Secondary)
    }
}