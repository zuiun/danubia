use crate::collections::OuterJoinMap;
use crate::common::{ID, ID_UNINITIALISED};
use std::collections::HashSet;

#[derive (Debug)]
pub struct Faction {
    id: ID,
    member_ids: HashSet<ID>,
    leader_followers: OuterJoinMap<ID, ID>,
}

impl Faction {
    pub fn new (id: ID) -> Self {
        let member_ids: HashSet<ID> = HashSet::new ();
        let leader_followers: OuterJoinMap<ID, ID> = OuterJoinMap::new ();

        Self { id, member_ids, leader_followers }
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

    pub fn get_followers (&self, unit_id: &ID) -> &HashSet<ID> {
        self.leader_followers.get_first (unit_id)
                .unwrap_or_else (|| panic! ("Followers not found for unit {}", unit_id))
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

    pub fn build (&self) -> Faction {
        Faction::new (self.id)
    }
}
