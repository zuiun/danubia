use std::rc::Rc;
use super::Tool;
use crate::Lists;
use crate::common::{DURATION_PERMANENT, Target, Timed, ID};
use crate::dynamic::{Appliable, Applier, Status};
use crate::map::Area;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Activity {
    Timed (u16, u16), // current, maximum
    Passive,
    Toggled (ID, ID), // status 1, status 2
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Skill {
    id: ID,
    status_id: ID,
    target: Target,
    area: Area,
    range: u8,
    activity: Activity
}

impl Skill {
    pub const fn new (id: ID, status_id: ID, target: Target, area: Area, range: u8, activity: Activity) -> Self {
        assert! (matches! (target,
            Target::This
            | Target::Ally
            | Target::Allies
        ));

        Self { id, status_id, target, area, range, activity }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn switch_status (&mut self) -> (ID, ID) {
        let status_id_old: ID = self.status_id;
        let status_id_new: ID = if let Activity::Toggled (t0, t1) = self.activity {
            if status_id_old == t0 {
                self.status_id = t1;

                t1
            } else if status_id_old == t1 {
                self.status_id = t0;

                t0
            } else {
                panic! ("Invalid status {}", status_id_old)
            }
        } else {
            panic! ("Invalid activity {:?}", self.activity)
        };

        (status_id_old, status_id_new)
    }

    pub fn is_timed (&self) -> bool {
        matches! (self.activity, Activity::Timed { .. })
    }

    pub fn is_passive (&self) -> bool {
        matches! (self.activity, Activity::Passive { .. })
    }

    pub fn is_toggle (&self) -> bool {
        matches! (self.activity, Activity::Toggled { .. })
    }

    pub fn get_status_id (&self) -> ID {
        self.status_id
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
        let status: Status = *lists.get_status (&self.status_id);

        status.try_yield_appliable (lists)
    }

    fn get_target (&self) -> Target {
        self.target
    }
}

impl Timed for Skill {
    fn get_duration (&self) -> u16 {
        match self.activity {
            Activity::Timed (c, _) => c,
            Activity::Passive => DURATION_PERMANENT,
            Activity::Toggled ( .. ) => DURATION_PERMANENT,
        }
    }

    fn decrement_duration (&mut self) -> bool {
        match self.activity {
            Activity::Timed (c, m) => {
                if c == 0 {
                    true
                } else {
                    let duration: u16 = c.saturating_sub (1);

                    self.activity = Activity::Timed (duration, m);

                    false
                }
            }
            Activity::Passive => false,
            Activity::Toggled ( .. ) => false,
        }
    }
}

#[cfg (test)]
pub mod tests {
    use super::*;
    use crate::tests::generate_lists;

    pub fn generate_skills () -> (Skill, Skill) {
        let lists = generate_lists ();
        let skill_0 = *lists.get_skill (&0);
        let skill_1 = *lists.get_skill (&1);

        (skill_0, skill_1)
    }

    #[test]
    fn skill_decrement_duration () {
        let (mut skill_0, mut skill_1) = generate_skills ();

        // Test timed skill
        assert! (!skill_0.decrement_duration ());
        assert_eq! (skill_0.get_duration (), 1);
        assert! (!skill_0.decrement_duration ());
        assert_eq! (skill_0.get_duration (), 0);
        assert! (skill_0.decrement_duration ());
        assert_eq! (skill_0.get_duration (), 0);
        // Test passive skill
        assert! (!skill_1.decrement_duration ());
        assert_eq! (skill_1.get_duration (), DURATION_PERMANENT);
        assert! (!skill_1.decrement_duration ());
        assert_eq! (skill_1.get_duration (), DURATION_PERMANENT);
    }
}
