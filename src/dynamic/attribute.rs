use super::{Adjustment, Appliable, Applier, Effect, Modifier, AppliableKind, Trigger};
use crate::common::{Capacity, DURATION_PERMANENT, ID, Target, Timed};
use std::rc::Rc;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Attribute {
    id: ID,
    kind: AppliableKind,
    trigger: Trigger,
    duration: Capacity,
    applier_id: Option<ID>,
}

impl Attribute {
    pub const fn new (id: ID, kind: AppliableKind, trigger: Trigger, duration: u16) -> Self {
        assert! (duration > 0);

        let duration: Capacity = if duration < DURATION_PERMANENT {
            Capacity::Quantity (duration, duration)
        } else {
            Capacity::Constant (DURATION_PERMANENT, DURATION_PERMANENT, DURATION_PERMANENT)
        };
        let applier_id: Option<ID> = None;

        Self { id, kind, trigger, duration, applier_id }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_kind (&self) -> AppliableKind {
        self.kind
    }

    pub fn get_trigger (&self) -> Trigger {
        self.trigger
    }
}

impl Appliable for Attribute {
    fn modifier (&self) -> Modifier {
        unimplemented! ()
    }

    fn effect (&self) -> Effect {
        unimplemented! ()
    }

    fn attribute (&self) -> Attribute {
        *self
    }

    fn kind (&self) -> AppliableKind {
        AppliableKind::Attribute (self.id)
    }

    fn get_adjustments (&self) -> &[Adjustment] {
        unimplemented! ()
    }

    fn can_stack_or_is_flat (&self) -> bool {
        false
    }

    fn get_applier_id (&self) -> Option<ID> {
        self.applier_id
    }

    fn set_applier_id (&mut self, applier_id: ID) {
        self.applier_id = Some (applier_id)
    }
}

impl Applier for Attribute {
    fn try_yield_appliable (&self, scene: Rc<crate::Scene>) -> Option<Box<dyn Appliable>> {
        let appliable: Box<dyn Appliable> = self.kind.appliable (scene);

        Some (appliable)
    }

    fn get_target (&self) -> Target {
        match self.trigger {
            Trigger::OnHit => Target::Enemy,
            Trigger::OnAttack => Target::Enemy,
            Trigger::OnOccupy => Target::Map (0),
            Trigger::None => Target::This,
        }
    }
}

impl Timed for Attribute {
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

impl PartialEq for Attribute {
    fn eq (&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg (test)]
mod tests {    
    use super::*;

    #[test]
    fn attribute_decrement_duration () {
        let mut attribute_0 = Attribute::new (0, AppliableKind::Modifier (0), Trigger::None, 2);
        let mut attribute_1 = Attribute::new (1, AppliableKind::Modifier (0), Trigger::None, DURATION_PERMANENT);

        // Test timed attribute
        assert! (attribute_0.decrement_duration ());
        assert_eq! (attribute_0.get_duration (), 1);
        assert! (attribute_0.decrement_duration ());
        assert_eq! (attribute_0.get_duration (), 0);
        assert! (!attribute_0.decrement_duration ());
        assert_eq! (attribute_0.get_duration (), 0);
        // Test permanent attribute
        assert! (attribute_1.decrement_duration ());
        assert_eq! (attribute_1.get_duration (), DURATION_PERMANENT);
        assert! (attribute_1.decrement_duration ());
        assert_eq! (attribute_1.get_duration (), DURATION_PERMANENT);
    }
}
