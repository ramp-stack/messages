use pelican_ui::events::OnEvent;
use pelican_ui::drawable::{Drawable, Component, Align};
use pelican_ui::layout::{Area, SizeRequest, Layout};
use pelican_ui::{Context, Component};

use profiles::plugin::ProfilePlugin;
use profiles::components::AvatarContentProfiles;
use pelican_ui::air::OrangeName;

use chrono::Duration;

use crate::service::Message;
use crate::components::AvatarMessages;

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
        mut style: MessageType,
        mut messages: Vec<String>,
        author: OrangeName,
        timestamp: Timestamp
    ) -> Self {
        if author == ProfilePlugin::me(ctx).0 { style = MessageType::You; }
        let username = ProfilePlugin::username(ctx, &author);
        let avatar_content = AvatarContentProfiles::from_orange_name(ctx, &author);

        let (offset, avatar) = match style {
            MessageType::You => (Offset::End, false),
            MessageType::Rooms => (Offset::Start, true),
            _ => (Offset::End, true),
        };

        TextMessage (
            Row::new(8.0, offset, Size::Fit, Padding::default()),
            avatar.then(|| AvatarMessages::new(ctx, avatar_content)),
            MessageContent::new(ctx, style, messages, &username, timestamp)
        )
    }

    fn content(&mut self) -> &mut MessageContent {&mut self.2}
}

#[derive(Debug, Component)]
struct MessageContent(Column, Option<MessageData>, MessageBubbles, Option<MessageData>);
impl OnEvent for MessageContent {}

impl MessageContent {
    fn new(
        ctx: &mut Context,
        style: MessageType,
        messages: Vec<String>,
        name: &str,
        time: Timestamp,
    ) -> Self {
        let name = match style {
            MessageType::You => None,
            _ => Some(name),
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
            top, MessageBubbles::new(ctx, messages, style), bottom
        )
    }

    fn bubbles(&mut self) -> &mut MessageBubbles {&mut self.2}
}

#[derive(Debug, Component)]
struct MessageData(Row, Option<Text>, Option<Text>, Text);
impl OnEvent for MessageData {}

impl MessageData {
    fn new(
        ctx: &mut Context,
        style: MessageType,
        name: Option<&str>,
        time: Timestamp,
    ) -> Self {
        let text_size = ctx.theme.fonts.size;
        let (title, divider) = match style {
            MessageType::Rooms => (Some((TextStyle::Heading, text_size.h5)), false),
            MessageType::Contact => (None, true),
            MessageType::Group => (Some((TextStyle::Secondary, text_size.sm)), true),
            MessageType::You => (None, true)
        };
        MessageData(
            Row::new(4.0, Offset::End, Size::Fit, Padding::default()),
            title.map(|(style, size)| name.map(|n| Text::new(ctx, n, style, size, Align::Left))).flatten(),
            divider.then(|| title.is_some().then(|| Text::new(ctx, "·", TextStyle::Secondary, text_size.sm, Align::Left))).flatten(),
            Text::new(ctx, &time.direct(), TextStyle::Secondary, text_size.sm, Align::Left),
        )
    }
}


#[derive(Debug, Component)]
struct MessageBubbles(Column, Vec<MessageBubble>);
impl OnEvent for MessageBubbles {}

impl MessageBubbles {
    fn new(
        ctx: &mut Context,
        messages: Vec<String>,
        // message: &str,
        style: MessageType,
    ) -> Self {
        let messages = messages.iter().map(|m| MessageBubble::new(ctx, m.as_str(), style)).collect();
        let offset = if style == MessageType::You { Offset::End } else { Offset::Start };
        MessageBubbles(Column::new(8.0, offset, Size::Fit, Padding::default()), messages)
    }

    fn bubbles(&mut self) -> &mut Vec<MessageBubble> {&mut self.1}
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
    pub fn new(ctx: &mut Context, messages: &Vec<Message>, style: MessageType) -> Self {
        let mut result = Vec::new();
        let mut section = Vec::new();
        let mut last_author = None;
        let mut last_time = None;

        for msg in messages {
            let time = msg.timestamp().to_datetime();
            let author = msg.author().clone();

            let same_author = Some(author.clone()) == last_author;
            let close_time = last_time.map(|t| (time - t) <= Duration::minutes(1)).unwrap_or(false);

            if same_author && close_time {
                section.push(msg.message().clone());
                last_time = Some(time);
            } else {
                if let (Some(author), Some(time)) = (last_author, last_time) {
                    result.push(TextMessage::new(ctx, style, section, author, Timestamp::new(time)));
                }
                section = vec![msg.message().clone()];
                last_author = Some(author);
                last_time = Some(time);
            }
        }

        if let (Some(author), Some(time)) = (last_author, last_time) {
            result.push(TextMessage::new(ctx, style, section, author, Timestamp::new(time)));
        }

        TextMessageGroup(Column::center(24.0), result)
    }

    pub fn count(&mut self) -> usize {
        self.1.iter_mut().map(|msg| msg.content().bubbles().bubbles().len()).sum()
    }
}
