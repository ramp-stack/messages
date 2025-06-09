use profiles::service::Profile;
use pelican_ui::events::Event;
use pelican_ui::Context;

use maverick_os::air::air;
use air::orange_name::OrangeName;

/// Event to add a contact to a `QuickDeselect` component.
#[derive(Debug, Clone)]
pub struct AddContactEvent(pub OrangeName);

impl Event for AddContactEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Event to remove a contact from a `QuickDeselect` component.
#[derive(Debug, Clone)]
pub struct RemoveContactEvent(pub OrangeName);

impl Event for RemoveContactEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Event to remove a contact from a `QuickDeselect` component.
#[derive(Debug, Clone)]
pub struct CreateMessageEvent(pub uuid::Uuid);

impl Event for CreateMessageEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}