use pelican_ui::Context;
use pelican_ui::drawable::Align;
use profiles::components::AvatarContentProfiles;
use pelican_ui::air::OrangeName;
use profiles::plugin::ProfilePlugin;

use pelican_ui_std::{
    IconButton, Header, AvatarRow, 
    Text, TextStyle, NavigateEvent,
    HeaderContent, HeaderIcon,
};


pub struct HeaderMessages;

impl HeaderMessages {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, profiles: Vec<OrangeName>) -> Header {
        let left = IconButton::navigation(ctx, "left", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(0)));
        let right = IconButton::navigation(ctx, "info", move |ctx: &mut Context| ctx.trigger_event(NavigateEvent(1)));
        Header::new(HeaderIcon::new(Some(left)), HeaderContentMessages::new(ctx, profiles), HeaderIcon::new(Some(right)))
    }
}

pub struct HeaderContentMessages;

impl HeaderContentMessages {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, mut profiles: Vec<OrangeName>) -> HeaderContent {
        let me = ProfilePlugin::me(ctx).0;
        profiles.retain(|p| *p != me);
        if profiles.is_empty() { profiles.push(me); }
        let profiles: Vec<_> = profiles.into_iter().map(|p| (ProfilePlugin::username(ctx, &p).to_string(), AvatarContentProfiles::from_orange_name(ctx, &p))).collect();
        let title = if profiles.len() == 1 { profiles[0].0.clone() } else { "Group Message".to_string() };
        let avatars = profiles.into_iter().map(|(_, a)| a).collect();

        HeaderContent::new(Some(AvatarRow::new(ctx, avatars)),
            Text::new(ctx, &title, TextStyle::Heading, ctx.theme.fonts.size.h5, Align::Left),
        )
    }
}