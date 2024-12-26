use super::{Adjustments, Capacity, DURATION_PERMANENT, ID, Timed};

pub trait Modifiable {
    fn add_modifier (&mut self, modifier: Modifier) -> bool;
    fn remove_modifier (&mut self, modifier_id: &ID) -> bool;
    fn dec_durations (&mut self) -> ();
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Modifier {
    id: ID,
    adjustments: Adjustments,
    duration: Capacity,
    can_stack: bool // for tiles: false = set to constant, true = flat change
}

impl Modifier {
    pub const fn new (id: ID, adjustments: Adjustments, duration: u16, can_stack: bool) -> Self {
        assert! (duration > 0);

        let duration: Capacity = if duration < DURATION_PERMANENT {
            Capacity::Quantity (duration, duration)
        } else {
            Capacity::Constant (DURATION_PERMANENT, DURATION_PERMANENT, DURATION_PERMANENT)
        };

        Self { id, adjustments, duration, can_stack }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_adjustments (&self) -> Adjustments {
        self.adjustments
    }

    pub fn can_stack (&self) -> bool {
        self.can_stack
    }
}

impl Timed for Modifier {
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

impl PartialEq for Modifier {
    fn eq (&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg (test)]
mod tests {    
    use super::*;
    use crate::engine::common::Statistic;

    #[test]
    fn modifier_get_duration () {
        let modifier_0: Modifier = Modifier::new (0,
            [None, None, None, None],
            2, true
        );
        let modifier_1: Modifier = Modifier::new (0,
            [None, None, None, None],
            DURATION_PERMANENT, true
        );

        // Test timed modifier
        assert_eq! (modifier_0.get_duration (), 2);
        // Test permanent modifier
        assert_eq! (modifier_1.get_duration (), DURATION_PERMANENT);
    }

    #[test]
    fn modifier_dec_duration () {
        let mut modifier_0: Modifier = Modifier::new (0,
            [Some ((Statistic::Tile, 1, false)), None, None, None],
            2, true
        );
        let mut modifier_1: Modifier = Modifier::new (0,
            [Some ((Statistic::Tile, 1, false)), None, None, None],
            DURATION_PERMANENT, true
        );

        // Test timed modifier
        assert_eq! (modifier_0.dec_duration (), false);
        assert_eq! (modifier_0.get_duration (), 1);
        assert_eq! (modifier_0.dec_duration (), true);
        assert_eq! (modifier_0.get_duration (), 0);
        assert_eq! (modifier_0.dec_duration (), true);
        // Test permanent modifier
        assert_eq! (modifier_1.dec_duration (), false);
        assert_eq! (modifier_1.get_duration (), DURATION_PERMANENT);
        assert_eq! (modifier_1.dec_duration (), false);
        assert_eq! (modifier_1.get_duration (), DURATION_PERMANENT);
    }
}
