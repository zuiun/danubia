use core::fmt::Debug;
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use crate::engine::common::{DuplicateCrossMap, ID};

pub type Flag = (usize, usize, usize);
pub type Event = (ID, Flag); // event, flag
pub type Response = (usize, usize);

// ID::MAX is reserved for notifications
pub const EVENT_UNIT_DIE: ID = 0; // notification
pub const EVENT_CITY_DRAW_SUPPLY: ID = 1; // value = city ID, unit ID
pub const EVENT_UNIT_DRAW_SUPPLY: ID = 2; // value = unit ID
pub const EVENT_CITY_STOCKPILE_SUPPLY: ID = 3; // value = unit ID

pub const FLAG_NOTIFICATION: Flag = (usize::MAX, usize::MAX, usize::MAX);

pub const RESPONSE_NOTIFICATION: Response = (usize::MAX, usize::MAX);

pub trait Observer: Debug {
    fn subscribe (&mut self, event_id: ID) -> ID;
    fn unsubscribe (&mut self, event_id: ID) -> ID;
    fn update (&mut self, event_id: ID) -> ();
    fn get_observer_id (&self) -> Option<ID>;
}

pub trait Subject {
    async fn notify (&self, event: Event) -> Response;
}

#[derive (Debug)]
pub struct Handler {
    observers: HashMap<ID, Rc<RefCell<dyn Observer>>>,
    event_observers: DuplicateCrossMap<ID, ID>,
    id: ID
}

impl Handler {
    pub fn new () -> Self {
        let observers: HashMap<ID, Rc<RefCell<dyn Observer>>> = HashMap::new ();
        let event_observers: DuplicateCrossMap<ID, ID> = DuplicateCrossMap::new ();
        let id: ID = 0;

        Self { observers, event_observers, id }
    }

    pub fn subscribe (&mut self, observer: Rc<RefCell<dyn Observer>>, event_id: ID) -> ID {
        let observer_id: ID = self.id;

        self.observers.insert (observer_id, observer);
        self.event_observers.insert ((event_id, observer_id));
        self.id += 1;

        observer_id
    }

    pub fn unsubscribe (&mut self, observer_id: &ID, event_id: &ID) -> Option<Rc<RefCell<dyn Observer>>> {
        let observer: Option<Rc<RefCell<dyn Observer>>> = self.observers.remove (observer_id);

        match observer {
            Some (o) => {
                let observer_id: ID = o.borrow ().get_observer_id ()?;

                self.event_observers.remove_second (&observer_id);

                Some (o)
            }
            None => None
        }
    }
}
