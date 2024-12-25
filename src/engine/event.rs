use core::fmt::Debug;
use std::{cell::RefCell, rc::Rc};
use crate::engine::common::ID;

pub type Event = (ID, (ID, ID), usize); // action, subject, flag (may be bit-packed)
pub type Result = (usize, usize);

// ID::MAX is reserved for notifications
pub const ACTION_UNIT_DIE: ID = 0; // notification
pub const ACTION_CITY_DRAW_SUPPLY: ID = 1; // value = city ID, unit ID
pub const ACTION_UNIT_DRAW_SUPPLY: ID = 2; // value = unit ID
pub const ACTION_CITY_STOCKPILE_SUPPLY: ID = 3; // value = unit ID

pub const OBSERVER_NOTIFICATION: (ID, ID) = (ID::MAX, ID::MAX);
pub const OBSERVER_ID: ID = ID::MAX;

pub const VALUE_NOTIFICATION: usize = usize::MAX;

pub const RESULT_NOTIFICATION: Result = (usize::MAX, usize::MAX);

pub trait Observer: Debug {
    fn update (&mut self, event: Event) -> ();
}

pub trait Subject {
    fn add_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> ();
    fn remove_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> ();
    async fn notify (&self, event: Event) -> Result;
}
