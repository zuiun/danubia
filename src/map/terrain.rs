use crate::common::ID;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Terrain {
    modifier_id: Option<ID>,
    cost: u8,
}

impl Terrain {
    pub const fn new (modifier_id: Option<ID>, cost: u8 ) -> Self {
        Self { modifier_id, cost }
    }

    pub fn get_modifier_id (&self) -> Option<ID> {
        self.modifier_id
    }

    pub fn get_cost (&self) -> u8 {
        self.cost
    }
}
