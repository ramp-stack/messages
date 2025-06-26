use pelican_ui::Context;
use pelican_ui_std::{IconButton, AppPage, IconButtonRow, NavigateEvent};

use std::sync::{Arc, Mutex};

use profiles::pages::Account;

pub struct IconButtonMessages;
impl IconButtonMessages {
    pub fn new(ctx: &mut Context) -> (&'static str, Box<dyn FnMut(&mut Context) -> Box<dyn AppPage>>) {
        let closure = Box::new(move |ctx: &mut Context| {
            // let mut rooms = ctx.state().get::<Rooms>();
            // for (id, room) in rooms.0.iter() {
            //     if room.authors.len() == 1 && room.authors[0] == orange_name {
            //         let (on_return, with_nav) =
            //             UserAccount::new(ctx, &orange_name, account_return.lock().unwrap().take().unwrap());
            //         let page = DirectMessage::new(ctx, id, (on_return.into_boxed(), with_nav));
            //         ctx.trigger_event(NavigateEvent::new(page));
            //         return;
            //     }
            // }

            // // Create new DM if none found
            // let id = uuid::Uuid::new_v4();
            // let (on_return, with_nav) = MessagesHome::new(ctx);
            // let (page, with_nav) = DirectMessage::new(ctx, &id, (on_return.into_boxed(), with_nav));
            // rooms.add(Room::new(vec![orange_name.clone()]), id);
            // ctx.state().set(&rooms);
            // ctx.trigger_event(NavigateEvent(Some(page.into_boxed()), with_nav));
            Box::new(Account::new(ctx)) as Box<dyn AppPage>
        });

        ("messages", closure)
    }
}