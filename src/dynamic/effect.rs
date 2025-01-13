use super::{Adjustment, Appliable, Attribute, Modifier, AppliableKind};
use crate::common::ID;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Effect {
    id: ID,
    adjustments: &'static [Adjustment],
    is_flat: bool,
}

impl Effect {
    pub const fn new (id: ID, adjustments: &'static [Adjustment], is_flat: bool) -> Self {
        Self { id, adjustments, is_flat }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }
}

impl Appliable for Effect {
    fn modifier (&self) -> Modifier {
        unimplemented! ()
    }

    fn effect (&self) -> Effect {
        *self
    }

    fn attribute (&self) -> Attribute {
        unimplemented! ()
    }

    fn kind (&self) -> AppliableKind {
        AppliableKind::Effect (self.id)   
    }

    fn get_adjustments (&self) -> &[Adjustment] {
        self.adjustments
    }

    fn can_stack_or_is_flat (&self) -> bool {
        self.is_flat
    }

    fn get_applier_id (&self) -> Option<ID> {
        unimplemented! ()
    }

    fn set_applier_id (&mut self, _applier_id: ID) {
        unimplemented! ()
    }
}
