use super::{Adjustment, Appliable, Attribute, Effect, AppliableKind};
use crate::common::{Capacity, Timed, DURATION_PERMANENT, ID, ID_UNINITIALISED};

const ADJUSTMENTS_EMPTY: &[Adjustment] = &[];

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Modifier {
    id: ID,
    adjustments: &'static [Adjustment],
    duration: Capacity,
    can_stack: bool,
    is_every_turn: bool,
    next_id: Option<ID>,
    applier_id: Option<ID>,
}

impl Modifier {
    pub const fn new (id: ID, adjustments: &'static [Adjustment], duration: u16, can_stack: bool, is_every_turn: bool, next_id: Option<ID>) -> Self {
        let duration: Capacity = if duration < DURATION_PERMANENT {
            Capacity::Quantity (duration, duration)
        } else {
            Capacity::Constant (DURATION_PERMANENT, DURATION_PERMANENT, DURATION_PERMANENT)
        };
        let applier_id: Option<ID> = None;

        Self { id, adjustments, duration, can_stack, is_every_turn, next_id, applier_id }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn is_every_turn (&self) -> bool {
        self.is_every_turn
    }

    pub fn get_next_id (&self) -> Option<ID> {
        self.next_id
    }

    pub fn set_is_every_turn (&mut self, is_every_turn: bool) {
        self.is_every_turn = is_every_turn;
    }
}

impl Appliable for Modifier {
    fn modifier (&self) -> Modifier {
        *self
    }

    fn effect (&self) -> Effect {
        unimplemented! ()
    }

    fn attribute (&self) -> Attribute {
        unimplemented! ()
    }

    fn kind (&self) -> AppliableKind {
        AppliableKind::Modifier (self.id)
    }

    fn get_adjustments (&self) -> &[Adjustment] {
        self.adjustments
    }

    fn can_stack_or_is_flat (&self) -> bool {
        self.can_stack
    }

    fn get_applier_id (&self) -> Option<ID> {
        self.applier_id
    }

    fn set_applier_id (&mut self, applier_id: ID) {
        self.applier_id = Some (applier_id);
    }
}

impl Timed for Modifier {
    fn get_duration (&self) -> u16 {
        match self.duration {
            Capacity::Constant ( .. ) => DURATION_PERMANENT,
            Capacity::Quantity (d, _) => d,
        }
    }

    fn decrement_duration (&mut self) -> bool {
        match self.duration {
            Capacity::Constant ( .. ) => true,
            Capacity::Quantity (d, m) => {
                if d > 0 {
                    let duration: u16 = d.saturating_sub (1);

                    self.duration = Capacity::Quantity (duration, m);

                    true
                } else {
                    false
                }
            }
        }
    }
}

impl PartialEq for Modifier {
    fn eq (&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for Modifier {
    fn default () -> Self {
        let id: ID = ID_UNINITIALISED;
        let adjustments: &'static [Adjustment] = ADJUSTMENTS_EMPTY;
        let duration: Capacity = Capacity::Constant (1, 0, 0);
        let can_stack: bool = false;
        let is_every_turn: bool = false;
        let next_id: Option<ID> = None;
        let applier_id: Option<ID> = None;

        Self { id, adjustments, duration, can_stack, is_every_turn, next_id, applier_id }
    }
}

#[cfg (test)]
mod tests {    
    use super::*;
    use crate::tests::generate_scene;

    fn generate_modifiers () -> (Modifier, Modifier) {
        let scene = generate_scene ();
        let modifier_0 = *scene.get_modifier (&0);
        let modifier_1 = *scene.get_modifier (&1);

        (modifier_0, modifier_1)
    }

    #[test]
    fn modifier_decrement_duration () {
        let (mut modifier_0, mut modifier_1) = generate_modifiers ();

        // Test timed modifier
        assert! (modifier_0.decrement_duration ());
        assert_eq! (modifier_0.get_duration (), 1);
        assert! (modifier_0.decrement_duration ());
        assert_eq! (modifier_0.get_duration (), 0);
        assert! (!modifier_0.decrement_duration ());
        assert_eq! (modifier_0.get_duration (), 0);
        // Test permanent modifier
        assert! (modifier_1.decrement_duration ());
        assert_eq! (modifier_1.get_duration (), DURATION_PERMANENT);
        assert! (modifier_1.decrement_duration ());
        assert_eq! (modifier_1.get_duration (), DURATION_PERMANENT);
    }
}
