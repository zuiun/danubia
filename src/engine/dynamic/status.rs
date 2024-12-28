use crate::engine::common::{Capacity, DURATION_PERMANENT, ID, Target, Timed};
use super::{Appliable, Applier, Change, Effect, Modifier, ModifierBuilder, Trigger};

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Status {
    change: Change,
    trigger: Trigger,
    duration: Capacity,
    target: Target,
    next_id: Option<ID>,
}

impl Status {
    pub const fn new (change: Change, trigger: Trigger, duration: u16, target: Target, next_id: Option<ID>) -> Self {
        assert! (duration > 0);
        assert! (matches! (target, Target::This) || matches! (target, Target::Enemy) || matches! (target, Target::Map));

        let duration: Capacity = if duration < DURATION_PERMANENT {
            Capacity::Quantity (duration, duration)
        } else {
            assert! (next_id.is_none ());

            Capacity::Constant (DURATION_PERMANENT, DURATION_PERMANENT, DURATION_PERMANENT)
        };

        Self { change, trigger, duration, target, next_id }
    }

    pub fn get_change (&self) -> Change {
        self.change
    }

    pub fn get_trigger (&self) -> Trigger {
        self.trigger
    }

    pub fn get_next_id (&self) -> Option<ID> {
        self.next_id
    }
}

impl Applier for Status {
    fn try_yield_appliable (&self, lists: std::rc::Rc<crate::engine::Lists>) -> Option<Box<dyn Appliable>> {
        match self.change {
            Change::Modifier (m, s) => {
                let modifier_builder: &ModifierBuilder = lists.get_modifier_builder (&m);
                let modifier: Modifier = modifier_builder.build (self.get_duration (), s);

                Some (Box::new (modifier))
            }
            Change::Effect (e) => {
                let effect: &Effect = lists.get_effect (&e);

                Some (Box::new (effect.clone ()))
            }
        }
    }

    fn get_target (&self) -> Option<Target> {
        Some (self.target)
    }
}

impl Timed for Status {
    fn get_duration (&self) -> u16 {
        match self.duration {
            Capacity::Constant (_, _, _) => DURATION_PERMANENT,
            Capacity::Quantity (d, _) => d
        }
    }

    fn dec_duration (&mut self) -> bool {
        match self.duration {
            Capacity::Constant (_, _, _) => false,
            Capacity::Quantity (d, m) => {
                let duration: u16 = d.checked_sub (1).unwrap_or (0);

                self.duration = Capacity::Quantity (duration, m);

                duration == 0
            }
        }
    }
}

#[cfg (test)]
mod tests {    
    use super::*;

    #[test]
    fn status_dec_duration () {
        let mut status_0: Status = Status::new (Change::Modifier (0, false), Trigger::None, 2, Target::This, None);
        let mut status_1: Status = Status::new (Change::Modifier (0, false), Trigger::None, DURATION_PERMANENT, Target::This, None);

        // Test timed status
        assert_eq! (status_0.dec_duration (), false);
        assert_eq! (status_0.get_duration (), 1);
        assert_eq! (status_0.dec_duration (), true);
        assert_eq! (status_0.get_duration (), 0);
        assert_eq! (status_0.dec_duration (), true);
        // Test permanent status
        assert_eq! (status_1.dec_duration (), false);
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
        assert_eq! (status_1.dec_duration (), false);
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
    }
}
