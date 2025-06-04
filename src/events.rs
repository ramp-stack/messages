use pelican_ui_profiles::Profile;
use pelican_ui::events::Event;
use pelican_ui::Context;

/// Event to add a contact to a `QuickDeselect` component.
#[derive(Debug, Clone)]
pub struct AddContactEvent(pub Profile);

impl Event for AddContactEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}

/// Event to remove a contact from a `QuickDeselect` component.
#[derive(Debug, Clone)]
pub struct RemoveContactEvent(pub Profile);

impl Event for RemoveContactEvent {
    fn pass(self: Box<Self>, _ctx: &mut Context, children: Vec<((f32, f32), (f32, f32))>) -> Vec<Option<Box<dyn Event>>> {
        children.into_iter().map(|_| Some(self.clone() as Box<dyn Event>)).collect()
    }
}