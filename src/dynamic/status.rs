use std::rc::Rc;

use crate::common::{Capacity, DURATION_PERMANENT, ID, Target, Timed};
use super::{Appliable, Applier, Change, Trigger};

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Status {
    id: ID,
    change: Change,
    trigger: Trigger,
    duration: Capacity,
    target: Target,
    is_every_turn: bool,
    next_id: Option<ID>,
}

impl Status {
    pub const fn new (id: ID, change: Change, trigger: Trigger, duration: u16, target: Target, is_every_turn: bool, next_id: Option<ID>) -> Self {
        assert! (duration > 0);
        assert! (matches! (target, Target::This) || matches! (target, Target::Enemy) || matches! (target, Target::Map));

        let duration: Capacity = if duration < DURATION_PERMANENT {
            Capacity::Quantity (duration, duration)
        } else {
            assert! (next_id.is_none ());

            Capacity::Constant (DURATION_PERMANENT, DURATION_PERMANENT, DURATION_PERMANENT)
        };

        Self { id, change, trigger, duration, target, is_every_turn, next_id }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_change (&self) -> Change {
        self.change
    }

    pub fn get_trigger (&self) -> Trigger {
        self.trigger
    }

    pub fn is_every_turn (&self) -> bool {
        self.is_every_turn
    }

    pub fn get_next_id (&self) -> Option<ID> {
        self.next_id
    }
}

impl Applier for Status {
    fn try_yield_appliable (&self, lists: Rc<crate::Lists>) -> Option<Box<dyn Appliable>> {
        let appliable: Box<dyn Appliable> = self.change.appliable (lists);

        Some (appliable)
    }

    fn get_target (&self) -> Target {
        self.target
    }
}

impl Timed for Status {
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

impl PartialEq for Status {
    fn eq (&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg (test)]
mod tests {    
    use super::*;

    #[test]
    fn status_dec_duration () {
        let mut status_0 = Status::new (0, Change::Modifier (0, false), Trigger::None, 2, Target::This, false,  None);
        let mut status_1 = Status::new (1, Change::Modifier (0, false), Trigger::None, DURATION_PERMANENT, Target::This, false, None);

        // Test timed status
        assert_eq! (status_0.dec_duration (), false);
        assert_eq! (status_0.get_duration (), 1);
        assert_eq! (status_0.dec_duration (), false);
        assert_eq! (status_0.get_duration (), 0);
        assert_eq! (status_0.dec_duration (), true);
        assert_eq! (status_0.get_duration (), 0);
        // Test permanent status
        assert_eq! (status_1.dec_duration (), false);
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
        assert_eq! (status_1.dec_duration (), false);
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
    }
}
