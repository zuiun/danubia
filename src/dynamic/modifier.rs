use crate::common::{Capacity, DURATION_PERMANENT, ID, Timed};
use super::{Adjustments, Appliable, Change, Effect};

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Modifier {
    id: ID,
    adjustments: Adjustments,
    duration: Capacity,
    can_stack: bool,
}

impl Modifier {
    pub const fn new (id: ID, adjustments: Adjustments, duration: Capacity, can_stack: bool) -> Self {
        Self { id, adjustments, duration, can_stack }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn can_stack (&self) -> bool {
        self.can_stack
    }
}

impl Appliable for Modifier {
    fn effect (&self) -> Effect {
        unimplemented! ()
    }

    fn modifier (&self) -> Modifier {
        Modifier::new (self.id, self.adjustments, self.duration, self.can_stack)
    }

    fn change (&self) -> Change {
        Change::Modifier (self.id, self.can_stack)
    }

    fn get_adjustments (&self) -> Adjustments {
        self.adjustments
    }

    fn can_stack_or_is_flat (&self) -> bool {
        self.can_stack
    }
}

impl Timed for Modifier {
    fn get_duration (&self) -> u16 {
        match self.duration {
            Capacity::Constant ( .. ) => DURATION_PERMANENT,
            Capacity::Quantity (d, _) => d,
        }
    }

    fn dec_duration (&mut self) -> bool {
        match self.duration {
            Capacity::Constant ( .. ) => false,
            Capacity::Quantity (d, m) => {
                if d == 0 {
                    true
                } else {
                    let duration: u16 = d.checked_sub (1).unwrap_or (0);

                    self.duration = Capacity::Quantity (duration, m);

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

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct ModifierBuilder {
    id: ID,
    adjustments: Adjustments,
    duration: u16,
}

impl ModifierBuilder {
    pub const fn new (id: ID, adjustments: Adjustments, duration: u16) -> Self {
        assert! (duration > 0);

        Self { id, adjustments, duration }
    }

    pub fn build (&self, can_stack: bool) -> Modifier {
        let duration: Capacity = if self.duration < DURATION_PERMANENT {
            Capacity::Quantity (self.duration, self.duration)
        } else {
            Capacity::Constant (DURATION_PERMANENT, DURATION_PERMANENT, DURATION_PERMANENT)
        };

        Modifier::new (self.id, self.adjustments, duration, can_stack)
    }
}

#[cfg (test)]
mod tests {    
    use super::*;
    use crate::tests::generate_lists;

    fn generate_modifiers () -> (Modifier, Modifier) {
        let lists = generate_lists ();
        let modifier_builder_0 = lists.get_modifier_builder (&0);
        let modifier_0 = modifier_builder_0.build (false);
        let modifier_builder_1 = lists.get_modifier_builder (&1);
        let modifier_1 = modifier_builder_1.build (false);

        (modifier_0, modifier_1)
    }

    #[test]
    fn modifier_dec_duration () {
        let (mut modifier_0, mut modifier_1) = generate_modifiers ();

        // Test timed modifier
        assert_eq! (modifier_0.dec_duration (), false);
        assert_eq! (modifier_0.get_duration (), 1);
        assert_eq! (modifier_0.dec_duration (), false);
        assert_eq! (modifier_0.get_duration (), 0);
        assert_eq! (modifier_0.dec_duration (), true);
        assert_eq! (modifier_0.get_duration (), 0);
        // Test permanent modifier
        assert_eq! (modifier_1.dec_duration (), false);
        assert_eq! (modifier_1.get_duration (), DURATION_PERMANENT);
        assert_eq! (modifier_1.dec_duration (), false);
        assert_eq! (modifier_1.get_duration (), DURATION_PERMANENT);
    }
}
