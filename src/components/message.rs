use pelican_ui::events::OnEvent;
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use pelican_ui_std::{
    Padding,
    Size,
    Offset,
    Stack,
    Text,
    RoundedRectangle,
    TextStyle,
    Column,
    Timestamp,
    Row,
    Avatar,
    AvatarContent,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MessageType {
    You,
    Contact,
    Group,
    Rooms,
}

#[derive(Debug, Component)]
pub struct TextMessage(Row, Option<Avatar>, MessageContent);

impl OnEvent for TextMessage {}

impl TextMessage {
    pub fn new(
        ctx: &mut Context,
        style: MessageType,
        message: &str,
        sender: (String, AvatarContent), // name, biography, identifier, avatar
        time: Timestamp,
    ) -> Self {
        let (offset, avatar) = match style {
            MessageType::You => (Offset::End, false),
            MessageType::Rooms => (Offset::Start, true),
            _ => (Offset::End, true),
        };

        TextMessage (
            Row::new(8.0, offset, Size::Fit, Padding::default()),
            avatar.then(|| Avatar::new(ctx, sender.1, None, false, 24.0, None)),
            MessageContent::new(ctx, style, message, &sender.0, time)
        )
    }
}

#[derive(Debug, Component)]
struct MessageContent(Column, Option<MessageData>, MessageBubbles, Option<MessageData>);
impl OnEvent for MessageContent {}

impl MessageContent {
    fn new(
        ctx: &mut Context,
        style: MessageType,
        message: &str,
        name: &str,
        time: Timestamp,
    ) -> Self {
        let name = match style {
            MessageType::You => "You",
            _ => name,
        };
        let data = MessageData::new(ctx, style, name, time);

        let offset = match style {
            MessageType::You => Offset::End,
            _ => Offset::Start,
        };

        let (top, bottom) = match style {
            MessageType::Rooms => (Some(data), None),
            _ => (None, Some(data))
        };

        MessageContent(
            Column::new(8.0, offset, Size::custom(|widths: Vec<(f32, f32)>| (widths[1].0, f32::MAX)), Padding::default()),
            top, MessageBubbles::new(ctx, message, style), bottom
        )
    }
}

#[derive(Debug, Component)]
struct MessageData(Row, Text, Option<Text>, Text);
impl OnEvent for MessageData {}

impl MessageData {
    fn new(
        ctx: &mut Context,
        style: MessageType,
        name: &str,
        time: Timestamp,
    ) -> Self {
        let text_size = ctx.theme.fonts.size;
        let (title_style, title_size, divider) = match style {
            MessageType::Rooms => (TextStyle::Heading, text_size.h5, false),
            _ => (TextStyle::Secondary, text_size.sm, true),
        };
        MessageData(
            Row::new(4.0, Offset::End, Size::Fit, Padding::default()),
            Text::new(ctx, name, title_style, title_size, Align::Left),
            divider.then(|| Text::new(ctx, "·", TextStyle::Secondary, text_size.sm, Align::Left)),
            Text::new(ctx, &time.friendly(), TextStyle::Secondary, text_size.sm, Align::Left),
        )
    }
}


#[derive(Debug, Component)]
struct MessageBubbles(Column, Vec<MessageBubble>);
impl OnEvent for MessageBubbles {}

impl MessageBubbles {
    fn new(
        ctx: &mut Context,
        // messages: Vec<&str>,
        message: &str,
        style: MessageType,
    ) -> Self {
        // let messages = messages.iter().map(|m| MessageBubble::new(ctx, m, style)).collect();
        MessageBubbles(Column::new(8.0, Offset::Start, Size::Fit, Padding::default()), vec![MessageBubble::new(ctx, message, style)])
    }
}

#[derive(Debug, Component)]
struct MessageBubble(Stack, RoundedRectangle, Text);
impl OnEvent for MessageBubble {}

impl MessageBubble {
    fn new(
        ctx: &mut Context,
        message: &str,
        style: MessageType,
    ) -> Self {
        let theme = &ctx.theme;
        let (colors, text_size) = (theme.colors, theme.fonts.size.md);
        let (bg_color, text_style) = match style {
            MessageType::You => (colors.brand.primary, TextStyle::White),
            MessageType::Rooms => (colors.background.primary, TextStyle::White),
            MessageType::Group => (colors.background.secondary, TextStyle::Primary),
            MessageType::Contact => (colors.background.secondary, TextStyle::Primary),
        };

        let (hp, vp) = (12.0, 12.0);
        let max_w = 300.0-(hp*2.0);
        let background = RoundedRectangle::new(0.0, 16.0, bg_color);
        let mut content = Text::new(ctx, message, text_style, text_size, Align::Left);
        content.text().width = Some(max_w);
        let layout = Stack(
            Offset::Center, Offset::Center, 
            Size::custom(move |widths: Vec<(f32, f32)>| {
                let size = (widths[1].1+(hp*2.)).min(max_w+(hp*2.));
                (size, size)
            }),
            Size::custom(move |heights: Vec<(f32, f32)>| (heights[1].0+vp, heights[1].1+vp)), 
            Padding::default()
        );

        MessageBubble(layout, background, content)
    }
}

#[derive(Debug, Component)]
pub struct TextMessageGroup(Column, Vec<TextMessage>);
impl OnEvent for TextMessageGroup {}

impl TextMessageGroup {
    pub fn new(messages: Vec<TextMessage>) -> Self {
        TextMessageGroup(Column::center(24.0), messages)
    }

    pub fn messages(&mut self) -> &mut Vec<TextMessage> { &mut self.1 }
}
