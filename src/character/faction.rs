use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Weak;
use crate::common::{ID, ID_UNINITIALISED};
use crate::event::{Handler, Message, Observer, Response, Subject};
use crate::join_map::OuterJoinMap;

#[derive (Debug)]
pub struct Faction {
    id: ID,
    // Safety guarantee: Only Faction can reference its own member_ids
    member_ids: RefCell<HashSet<ID>>,
    leader_followers: RefCell<OuterJoinMap<ID, ID>>,
    // Safety guarantee: Faction will never borrow_mut Handler
    handler: Weak<RefCell<Handler>>,
    observer_id: Cell<ID>,
}

impl Faction {
    pub fn new (id: ID, handler: Weak<RefCell<Handler>>) -> Self {
        let member_ids: HashSet<ID> = HashSet::new ();
        let member_ids: RefCell<HashSet<ID>> = RefCell::new (member_ids);
        let leader_followers: OuterJoinMap<ID, ID> = OuterJoinMap::new ();
        let leader_followers: RefCell<OuterJoinMap<ID, ID>> = RefCell::new (leader_followers);
        let observer_id: Cell<ID> = Cell::new (ID_UNINITIALISED);

        Self { id, member_ids, leader_followers, handler, observer_id }
    }

    pub fn is_member (&self, unit_id: &ID) -> bool {
        self.member_ids.borrow ().contains (unit_id)
    }

    fn add_member (&self, unit_id: ID) -> bool {
        self.member_ids.borrow_mut ().insert (unit_id)
    }

    pub fn add_follower (&self, follower_id: ID, leader_id: ID) -> bool {
        self.leader_followers.borrow_mut ().insert ((leader_id, follower_id))
    }

    pub fn get_leader (&self, unit_id: &ID) -> ID {
        *self.leader_followers.borrow ().get_second (unit_id)
                .expect (&format! ("Leader not found for unit {}", unit_id))
    }

    pub fn get_followers (&self, unit_id: &ID) -> Vec<ID> {
        self.leader_followers.borrow ().get_first (unit_id)
                .expect (&format! ("Followers not found for unit {}", unit_id))
                .iter ()
                .map (|u: &ID| *u)
                .collect::<Vec<ID>> ()
    }
}

impl Observer for Faction {
    fn respond (&self, message: Message) -> Option<Response> {
        match message {
            Message::FactionIsMember (f, u) => if f == self.id {
                let is_member: bool = self.is_member (&u);

                Some (Response::FactionIsMember (is_member))
            } else {
                None
            }
            Message::FactionAddMember (f, u) => if f == self.id {
                self.add_member (u);

                Some (Response::FactionAddMember)
            } else {
                None
            }
            Message::FactionAddFollower (f, fol, l) => if f == self.id {
                self.add_follower (fol, l);

                Some (Response::FactionAddFollower)
            } else {
                None
            }
            Message::FactionGetLeader (f, u) => if f == self.id {
                let leader_id: ID = self.get_leader (&u);

                Some (Response::FactionGetLeader (leader_id))
            } else {
                None
            }
            Message::FactionGetFollowers (f, u) => if f == self.id {
                let follower_ids: Vec<ID> = self.get_followers (&u);

                Some (Response::FactionGetFollowers (follower_ids))
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
