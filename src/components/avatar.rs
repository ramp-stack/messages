use pelican_ui::{Context, Component};
use pelican_ui::drawable::{Component, Drawable};
use pelican_ui::layout::{SizeRequest, Area, Layout};
use pelican_ui::events::OnEvent;
use pelican_ui::resources;

use pelican_ui_std::{Avatar, AvatarContent, AvatarIconStyle, Row};

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


#[derive(Debug, Component)]
pub struct AvatarRow(Row, Vec<Avatar>);

impl OnEvent for AvatarRow {}

impl AvatarRow {
    pub fn new(ctx: &mut Context, avatars: Vec<AvatarContent>) -> Self {
        AvatarRow(
            Row::center(-16.0),
            avatars.into_iter().take(5).map(|avatar| Avatar::new(ctx, avatar, None, true, 32.0, None)).collect()
        )
    }

    pub fn update(&mut self, ctx: &mut Context, avatars: Vec<AvatarContent>) {
        self.1 = avatars.into_iter().take(5).map(|avatar| Avatar::new(ctx, avatar, None, true, 32.0, None)).collect()
    }

    pub fn count(&mut self) -> usize {self.1.len()}
}
