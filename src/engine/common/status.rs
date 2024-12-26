use super::{Capacity, DURATION_PERMANENT, ID, Target, Timed};

#[derive (Debug)]
pub struct Status {
    modifier_id: ID,
    // trigger: Condition, // TODO: triggered statuses: on hit -> reflect damage, on attack -> apply modifier, against specific units -> apply modifier, what else?
    duration: Capacity,
    target: Target,
    next: Option<Box<Status>>
}

impl Status {
    pub const fn new (modifier_id: ID, duration: u16, target: Target, next: Option<Box<Status>>) -> Self {
        assert! (duration > 0);

        let duration: Capacity = if duration < DURATION_PERMANENT {
            Capacity::Quantity (duration, duration)
        } else {
            Capacity::Constant (DURATION_PERMANENT, DURATION_PERMANENT, DURATION_PERMANENT)
        };

        Self { modifier_id, duration, target, next }
    }

    pub fn get_modifier_id (&self) -> ID {
        self.modifier_id
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
    fn status_get_duration () {
        let status_0: Status = Status::new (0, 2, Target::All (false), None);
        let status_1: Status = Status::new (0, DURATION_PERMANENT, Target::All (false), None);

        // Test timed modifier
        assert_eq! (status_0.get_duration (), 2);
        // Test permanent modifier
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
    }

    #[test]
    fn status_dec_duration () {
        let mut status_0: Status = Status::new (0, 2, Target::All (false), None);
        let mut status_1: Status = Status::new (0, DURATION_PERMANENT, Target::All (false), None);

        // Test timed modifier
        assert_eq! (status_0.dec_duration (), false);
        assert_eq! (status_0.get_duration (), 1);
        assert_eq! (status_0.dec_duration (), true);
        assert_eq! (status_0.get_duration (), 0);
        assert_eq! (status_0.dec_duration (), true);
        // Test permanent modifier
        assert_eq! (status_1.dec_duration (), false);
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
        assert_eq! (status_1.dec_duration (), false);
        assert_eq! (status_1.get_duration (), DURATION_PERMANENT);
    }
}
