use std::fmt;
use crate::common::ID;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Terrain {
    modifier_id: ID,
    cost: u8,
}

impl Terrain {
    pub const fn new (modifier_id: ID, cost: u8 ) -> Self {
        Self { modifier_id, cost }
    }

    pub fn get_modifier_id (&self) -> ID {
        self.modifier_id
    }

    pub fn get_cost (&self) -> u8 {
        self.cost
    }
}

impl fmt::Display for Terrain {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}", self.cost)
    }
}
