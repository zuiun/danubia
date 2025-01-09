use crate::common::ID;
use std::cmp::Ordering;

#[derive (Debug)]
pub struct Turn {
    unit_id: ID,
    delay: u8,
    mov: u16,
}

impl Turn {
    pub fn new (unit_id: ID, delay: u8, mov: u16) -> Self {
        Self { unit_id, delay, mov }
    }

    pub fn update (&mut self, delay: u8, mov: u16) -> bool {
        if let Some (d) = self.delay.checked_add (delay) {
            self.delay = d;
            self.mov = mov;

            true
        } else {
            false
        }
    }

    pub fn reduce_delay (&mut self, reduction: u8) -> u8 {
        self.delay -= reduction;

        self.delay
    }

    pub fn get_unit_id (&self) -> ID {
        self.unit_id
    }

    pub fn get_delay (&self) -> u8 {
        self.delay
    }
}

impl PartialEq for Turn {
    fn eq (&self, other: &Self) -> bool {
        self.unit_id == other.unit_id
    }
}

impl Eq for Turn {}

impl PartialOrd for Turn {
    fn partial_cmp (&self, other: &Self) -> Option<Ordering> {
        Some (self.cmp (other))
    }
}

impl Ord for Turn {
    fn cmp (&self, other: &Self) -> Ordering {
        self.delay.cmp (&other.delay)
                .then_with (|| other.mov.cmp (&self.mov))
                .then_with (|| self.unit_id.cmp (&other.unit_id))
                .reverse ()
    }
}
