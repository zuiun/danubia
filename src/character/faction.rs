use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Weak;
use crate::collections::OuterJoinMap;
use crate::common::{ID, ID_UNINITIALISED};
use crate::event::Handler;

#[derive (Debug)]
pub struct Faction {
    id: ID,
    member_ids: HashSet<ID>,
    leader_followers: OuterJoinMap<ID, ID>,
    // Safety guarantee: Faction will never borrow_mut Handler
    handler: Weak<RefCell<Handler>>,
    // observer_id: Cell<ID>,
}

impl Faction {
    pub fn new (id: ID, handler: Weak<RefCell<Handler>>) -> Self {
        let member_ids: HashSet<ID> = HashSet::new ();
        let leader_followers: OuterJoinMap<ID, ID> = OuterJoinMap::new ();
        // let observer_id: Cell<ID> = Cell::new (ID_UNINITIALISED);

        Self { id, member_ids, leader_followers, handler/*, observer_id*/ }
    }

    pub fn is_member (&self, unit_id: &ID) -> bool {
        self.member_ids.contains (unit_id)
    }

    pub fn add_member (&mut self, unit_id: ID) -> bool {
        self.member_ids.insert (unit_id)
    }

    pub fn add_follower (&mut self, follower_id: ID, leader_id: ID) -> bool {
        if leader_id < ID_UNINITIALISED {
            self.leader_followers.insert ((leader_id, follower_id))
        } else {
            false
        }
    }

    pub fn remove_follower (&mut self, unit_id: &ID) -> bool {
        self.leader_followers.remove (unit_id)
    }

    pub fn get_leader (&self, unit_id: &ID) -> &ID {
        self.leader_followers.get_second (unit_id)
                .unwrap_or_else (|| panic! ("Leader not found for unit {}", unit_id))
    }

    pub fn get_followers (&self, unit_id: &ID) -> Vec<ID> {
        self.leader_followers.get_first (unit_id)
                .unwrap_or_else (|| panic! ("Followers not found for unit {}", unit_id))
                .iter ()
                .copied ()
                .collect::<Vec<ID>> ()
    }
}

// impl Observer for Faction {
//     fn respond (&self, message: Message) -> Option<Response> {
//         match message {
//             _ => None
//         }
//     }

//     fn set_observer_id (&self, observer_id: ID) -> bool {
//         if self.observer_id.get () < ID_UNINITIALISED {
//             false
//         } else {
//             self.observer_id.replace (observer_id);

//             true
//         }
//     }
// }

// impl Subject for Faction {
//     fn notify (&self, message: Message) -> Vec<Response> {
//         self.handler.upgrade ()
//                 .expect (&format! ("Pointer upgrade failed for {:?}", self.handler))
//                 .borrow ()
//                 .notify (message)
//     }
// }

#[derive (Debug)]
pub struct FactionBuilder {
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
