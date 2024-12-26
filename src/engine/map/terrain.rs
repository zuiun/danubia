use std::fmt;
use crate::engine::common::Modifier;

#[derive (Debug)]
pub struct Terrain {
    modifiers: Vec<Modifier>,
    cost: u8
}

impl Terrain {
    pub const fn new (modifiers: Vec<Modifier>, cost: u8 ) -> Self {
        Self { modifiers, cost }
    }

    pub fn get_modifiers (&self) -> &Vec<Modifier> {
        &self.modifiers
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
