use std::cell::Cell;
use std::rc::Rc;
use crate::Lists;
use crate::common::{Capacity, DURATION_PERMANENT, Target, Timed, ID};
use crate::dynamic::{Appliable, Applier, Status};
use crate::map::Area;
use super::Tool;

const STATUS_0: usize = 0;
const STATUS_1: usize = 1;
const TOGGLE_OFF: u16 = 0;
const TOGGLE_ON: u16 = 1;
const TOGGLE_0: Capacity = Capacity::Constant (TOGGLE_OFF, TOGGLE_ON, TOGGLE_OFF);
const TOGGLE_1: Capacity = Capacity::Constant (TOGGLE_OFF, TOGGLE_OFF, TOGGLE_ON);

#[derive (Debug)]
#[derive (Clone)]
pub struct Skill {
    id: ID,
    status_ids: [ID; 2],
    status_active: Cell<usize>,
    target: Target,
    area: Area,
    range: u8,
    cooldown: Cell<Capacity>, // constant: passive (passive, toggle one, toggle two), quantity: active (duration, maximum)
}

impl Skill {
    pub const fn new (id: ID, status_ids: [ID; 2], target: Target, area: Area, range: u8, cooldown: Capacity) -> Self {
        match cooldown {
            Capacity::Constant (TOGGLE_ON, TOGGLE_OFF, TOGGLE_OFF) => assert! (true),
            Capacity::Constant (TOGGLE_OFF, TOGGLE_ON, TOGGLE_OFF) => assert! (true),
            Capacity::Constant (TOGGLE_OFF, TOGGLE_OFF, TOGGLE_ON) => assert! (true),
            Capacity::Quantity (c, m) => assert! (c <= m),
            _ => assert! (false),
        }

        let status_active: usize = STATUS_0;
        let status_active: Cell<usize> = Cell::new (status_active);
        let cooldown: Cell<Capacity> = Cell::new (cooldown);

        Self { id, status_ids, status_active, target, area, range, cooldown }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn switch_status (&self) -> (ID, ID) {
        let status_id_old: ID = self.status_ids[self.status_active.get ()];
        let cooldown: Capacity = self.cooldown.get ();

        match cooldown {
            Capacity::Constant (TOGGLE_OFF, t0, t1) => {
                let status_active: usize = self.status_active.get ();

                if status_active == STATUS_0 {
                    assert_eq! (t0, TOGGLE_ON);
                    assert_eq! (t1, TOGGLE_OFF);

                    self.cooldown.replace (TOGGLE_1);
                    self.status_active.replace (STATUS_1);
                } else {
                    assert_eq! (t0, TOGGLE_OFF);
                    assert_eq! (t1, TOGGLE_ON);

                    self.cooldown.replace (TOGGLE_0);
                    self.status_active.replace (STATUS_0);
                }

                (status_id_old, self.get_status_id ())
            }
            _ => (self.get_status_id (), self.get_status_id ()),
        }
    }

    pub fn is_active (&self) -> bool {
        if let Capacity::Quantity ( .. ) = self.cooldown.get () {
            true
        } else {
            false
        }
    }

    pub fn is_passive (&self) -> bool {
        if let Capacity::Constant (TOGGLE_ON, TOGGLE_OFF, TOGGLE_OFF) = self.cooldown.get () {
            true
        } else {
            false
        }
    }

    pub fn is_toggle (&self) -> bool {
        if let Capacity::Constant (TOGGLE_OFF, _, _) = self.cooldown.get () {
            true
        } else {
            false
        }
    }

    pub fn get_status_id (&self) -> ID {
        self.status_ids[self.status_active.get ()]
    }
}

impl Tool for Skill {
    fn get_area (&self) -> Area {
        self.area
    }

    fn get_range (&self) -> u8 {
        self.range
    }
}

impl Applier for Skill {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        let status: Status = lists.get_status (&self.status_ids[self.status_active.get ()]).clone ();

        status.try_yield_appliable (lists)
    }

    fn get_target (&self) -> Target {
        self.target
    }
}

impl Timed for Skill {
    fn get_duration (&self) -> u16 {
        match self.cooldown.get () {
            Capacity::Constant ( .. ) => DURATION_PERMANENT,
            Capacity::Quantity (d, _) => d,
        }
    }

    fn dec_duration (&mut self) -> bool {
        let cooldown: Capacity = self.cooldown.get ();

        match cooldown {
            Capacity::Constant ( .. ) => false,
            Capacity::Quantity (d, m) => {
                if d == 0 {
                    true
                } else {
                    let duration: u16 = d.checked_sub (1).unwrap_or (0);

                    self.cooldown.replace (Capacity::Quantity (duration, m));

                    false
                }
            }
        }
    }
}

#[cfg (test)]
pub mod tests {
    use super::*;
    use crate::tests::generate_lists;

    pub fn generate_skills () -> (Skill, Skill) {
        let lists = generate_lists ();
        let skill_0 = lists.get_skill (&0).clone ();
        let skill_1 = lists.get_skill (&1).clone ();

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
