use super::Unit;
use crate::collections::OuterJoinMap;
use crate::common::{ID, ID_UNINITIALISED};
use std::collections::HashSet;

#[derive (Debug)]
pub struct Faction {
    id: ID,
    member_ids: HashSet<ID>,
    leader_followers: OuterJoinMap<ID, ID>,
    allies: &'static [ID],
}

impl Faction {
    pub fn new (id: ID, allies: &'static [ID], units: &[Unit]) -> Self {
        let mut member_ids: HashSet<ID> = HashSet::new ();
        let mut leader_followers: OuterJoinMap<ID, ID> = OuterJoinMap::new ();

        for unit in units {
            if unit.get_faction_id () == id {
                let unit_id: ID = unit.get_id ();
                let leader_id: ID = unit.get_leader_id ();

                member_ids.insert (unit_id);

                if leader_id < ID_UNINITIALISED {
                    leader_followers.insert ((leader_id, unit_id));
                }
            }
        }

        Self { id, member_ids, leader_followers, allies }
    }

    pub fn is_member (&self, unit_id: &ID) -> bool {
        self.member_ids.contains (unit_id)
    }

    pub fn is_ally (&self, faction_id: &ID) -> bool {
        *faction_id == self.id || self.allies.contains (faction_id)
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
    allies: &'static [ID],
}

impl FactionBuilder {
    pub const fn new (id: ID, allies: &'static [ID]) -> Self {
        Self { id, allies }
    }

    pub fn build (&self, units: &[Unit]) -> Faction {
        Faction::new (self.id, self.allies, units)
    }
}
