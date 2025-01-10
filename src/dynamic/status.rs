use super::{Appliable, Applier, Change, Trigger};
use crate::common::{Capacity, DURATION_PERMANENT, ID, Target, Timed};
use std::rc::Rc;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Status {
    id: ID,
    change: Change,
    trigger: Trigger,
    duration: Capacity,
    target: Target,
    is_every_turn: bool,
    is_expired: bool,
    next_id: Option<ID>,
}

impl Status {
    pub const fn new (id: ID, change: Change, trigger: Trigger, duration: u16, target: Target, is_every_turn: bool, next_id: Option<ID>) -> Self {
        assert! (duration > 0);
        assert! (matches! (target, Target::This) || matches! (target, Target::Enemy) || matches! (target, Target::Map ( .. )));

        let duration: Capacity = if duration < DURATION_PERMANENT {
            Capacity::Quantity (duration, duration)
        } else {
            assert! (next_id.is_none ());

            Capacity::Constant (DURATION_PERMANENT, DURATION_PERMANENT, DURATION_PERMANENT)
        };
        let is_expired: bool = false;

        Self { id, change, trigger, duration, target, is_every_turn, is_expired, next_id }
    }

    // pub fn replace (&mut self, other: &Self) {
    //     self.id = other.id;
    //     self.change = other.change;
    //     self.trigger = other.trigger;
    //     self.duration = other.duration;
    //     self.target = other.target;
    //     self.is_every_turn = other.is_every_turn;
    //     self.next_id = other.next_id;
    // }

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

    pub fn is_expired (&self) -> bool {
        self.is_expired
    }

    pub fn get_next_id (&self) -> Option<ID> {
        self.next_id
    }

    pub fn set_applier_id (&mut self, unit_id: ID) {
        if let Target::Map ( .. ) = self.target {
            self.target = Target::Map (unit_id);
        }
    }
}

impl Applier for Status {
    fn try_yield_appliable (&self, scene: Rc<crate::Scene>) -> Option<Box<dyn Appliable>> {
        let appliable: Box<dyn Appliable> = self.change.appliable (scene);

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

    fn decrement_duration (&mut self) -> bool {
        match self.duration {
            Capacity::Constant ( .. ) => false,
            Capacity::Quantity (d, m) => {
                if d == 0 {
                    self.is_expired = true;

                    true
                } else {
                    let duration: u16 = d.saturating_sub (1);

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
    fn status_decrement_duration () {
        let mut status_0 = Status::new (0, Change::Modifier (0, false), Trigger::None, 2, Target::This, false,  None);
        let mut status_1 = Status::new (1, Change::Modifier (0, false), Trigger::None, DURATION_PERMANENT, Target::This, false, None);

        // Test timed status
        assert! (!status_0.decrement_duration ());
        assert_eq! (status_0.get_duration (), 1);
        assert! (!status_0.decrement_duration ());
        assert_eq! (status_0.get_duration (), 0);
        assert! (status_0.decrement_duration ());
        assert_eq! (status_0.get_duration (), 0);
        // Test permanent status
        assert! (!status_1.decrement_duration ());
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
        assert! (!status_1.decrement_duration ());
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
    }
}
