use super::{Adjustments, Appliable, Change, Modifier};
use crate::common::ID;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Effect {
    id: ID,
    adjustments: Adjustments,
    is_flat: bool,
}

impl Effect {
    pub const fn new (id: ID, adjustments: Adjustments, is_flat: bool) -> Self {
        Self { id, adjustments, is_flat }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }
}

impl Appliable for Effect {
    fn effect (&self) -> Effect {
        Effect::new (self.id, self.adjustments, self.is_flat)
    }

    fn modifier (&self) -> Modifier {
        unimplemented! ()
    }

    fn change (&self) -> Change {
        Change::Effect (self.id)   
    }

    fn get_adjustments (&self) -> Adjustments {
        self.adjustments
    }

    fn can_stack_or_is_flat (&self) -> bool {
        self.is_flat
    }
}
