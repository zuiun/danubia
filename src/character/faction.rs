use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Weak;
use crate::common::{ID, ID_UNINITIALISED};
use crate::event::{Handler, Message, Observer, Response, Subject};

#[derive (Debug)]
pub struct Faction {
    id: ID,
    // Safety guarantee: Only Faction can reference its own member_ids
    member_ids: RefCell<HashSet<ID>>,
    // Safety guarantee: Faction will never borrow_mut Handler
    handler: Weak<RefCell<Handler>>,
    observer_id: Cell<ID>,
}

impl Faction {
    pub fn new (id: ID, handler: Weak<RefCell<Handler>>) -> Self {
        let member_ids: HashSet<ID> = HashSet::new ();
        let member_ids: RefCell<HashSet<ID>> = RefCell::new (member_ids);
        let observer_id: Cell<ID> = Cell::new (ID_UNINITIALISED);

        Self { id, member_ids, handler, observer_id }
    }

    fn is_member (&self, unit_id: &ID) -> bool {
        self.member_ids.borrow ().contains (unit_id)
    }

    fn add_member (&self, unit_id: ID) -> bool {
        self.member_ids.borrow_mut ().insert (unit_id)
    }
}

impl Observer for Faction {
    fn respond (&self, message: Message) -> Option<Response> {
        match message {
            Message::FactionIsMember (f, u) => if f == self.id {
                Some (Response::FactionIsMember (self.is_member (&u)))
            } else {
                None
            }
            Message::FactionAddMember (f, u) => if f == self.id {
                Some (Response::FactionAddMember (self.add_member (u)))
            } else {
                None
            }
            _ => None
        }
    }

    fn set_observer_id (&self, observer_id: ID) -> bool {
        if self.observer_id.get () < ID_UNINITIALISED {
            false
        } else {
            self.observer_id.replace (observer_id);

            true
        }
    }
}

impl Subject for Faction {
    fn notify (&self, message: Message) -> Vec<Response> {
        self.handler.upgrade ()
                .expect (&format! ("Pointer upgrade failed for {:?}", self.handler))
                .borrow ()
                .notify (message)
    }
}

#[derive (Debug)]
pub struct FactionBuilder {
    // TODO: ???
    id: ID,
}

impl FactionBuilder {
    pub const fn new (id: ID) -> Self {
        Self { id }
    }

    pub fn build (&self, handler: Weak<RefCell<Handler>>) -> Faction {
        Faction::new (self.id, handler)
    }
}
