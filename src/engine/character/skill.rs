use std::matches;
use std::rc::Rc;
use crate::engine::Lists;
use crate::engine::common::{Area, Capacity, DURATION_PERMANENT, Target, Timed, ID};
use crate::engine::dynamic::{Appliable, Applier, Status};

const STATUS_0: usize = 0;
const STATUS_1: usize = 1;
const TOGGLE_OFF: u16 = 0;
const TOGGLE_ON: u16 = 1;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Skill {
    status_ids: [ID; 2],
    status_active: usize,
    target: Target,
    area: Area,
    range: u8,
    cooldown: Capacity, // constant: passive (passive, toggle one, toggle two), quantity: active (duration, maximum)
}

impl Skill {
    pub const fn new (status_ids: [ID; 2], target: Target, area: Area, range: u8, duration: Capacity) -> Self {
        assert! (matches! (duration, Capacity::Constant (1, 0, 0)) || matches! (duration, Capacity::Constant (0, _, _)) || matches! (duration, Capacity::Quantity { .. }));

        let status_active: usize = STATUS_0;

        Self { status_ids, status_active, target, area, range, cooldown: duration }
    }

    pub fn switch_status (&mut self) -> bool {
        match self.cooldown {
            Capacity::Constant (TOGGLE_OFF, t1, t2) => {
                let mut toggle_first: u16 = TOGGLE_OFF;
                let mut toggle_second: u16 = TOGGLE_OFF;

                self.status_active = if self.status_active == STATUS_0 {
                    assert_eq! (t1, TOGGLE_ON);
                    assert_eq! (t2, TOGGLE_OFF);

                    toggle_second = TOGGLE_ON;

                    STATUS_1
                } else {
                    assert_eq! (t1, TOGGLE_OFF);
                    assert_eq! (t2, TOGGLE_ON);

                    toggle_first = TOGGLE_ON;

                    STATUS_0
                };
                self.cooldown = Capacity::Constant (0, toggle_first, toggle_second);

                true
            }
            _ => false,
        }
    }

    pub fn get_status_ids (&self) -> &[ID; 2] {
        &self.status_ids
    }

    fn get_area (&self) -> Area {
        self.area
    }

    fn get_range (&self) -> u8 {
        self.range
    }
}

impl Applier for Skill {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        let status: Status = lists.get_status (&self.status_ids[self.status_active]).clone ();

        status.try_yield_appliable (lists)
    }

    fn get_target (&self) -> Option<Target> {
        Some (self.target)
    }
}

impl Timed for Skill {
    fn get_duration (&self) -> u16 {
        match self.cooldown {
            Capacity::Constant (_, _, _) => DURATION_PERMANENT,
            Capacity::Quantity (d, _) => d,
        }
    }

    fn dec_duration (&mut self) -> bool {
        match self.cooldown {
            Capacity::Constant (_, _, _) => false,
            Capacity::Quantity (d, m) => {
                if d == 0 {
                    true
                } else {
                    let duration: u16 = d.checked_sub (1).unwrap_or (0);

                    self.cooldown = Capacity::Quantity (duration, m);

                    false
                }
            }
        }
    }
}

#[cfg (test)]
pub mod tests {
    use super::*;
    use crate::engine::tests::generate_lists;

    pub fn generate_skills () -> (Skill, Skill) {
        let lists: Rc<Lists> = generate_lists ();
        let skill_0: Skill = lists.get_skill (&0).clone ();
        let skill_1: Skill = lists.get_skill (&1).clone ();

        (skill_0, skill_1)
    }

    #[test]
    fn skill_dec_duration () {
        let (mut skill_0, mut skill_1) = generate_skills ();

        // Test active skill
        assert_eq! (skill_0.dec_duration (), false);
        assert_eq! (skill_0.get_duration (), 1);
        assert_eq! (skill_0.dec_duration (), false);
        assert_eq! (skill_0.get_duration (), 0);
        assert_eq! (skill_0.dec_duration (), true);
        assert_eq! (skill_0.get_duration (), 0);
        // Test passive skill
        assert_eq! (skill_1.dec_duration (), false);
        assert_eq! (skill_1.get_duration (), DURATION_PERMANENT);
        assert_eq! (skill_1.dec_duration (), false);
        assert_eq! (skill_1.get_duration (), DURATION_PERMANENT);
    }
}
