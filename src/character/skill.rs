use super::Tool;
use crate::common::{DURATION_PERMANENT, ID, Target, Timed};
use crate::dynamic::{Appliable, Applier, Status};
use crate::map::Area;
use crate::Scene;
use std::rc::Rc;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum SkillKind {
    Timed (u16, u16), // current, maximum
    Passive,
    Toggled (ID), // active status index
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Skill {
    id: ID,
    status_ids: &'static [ID],
    target: Target,
    area: Area,
    range: u8,
    kind: SkillKind,
}

impl Skill {
    pub const fn new (id: ID, status_ids: &'static [ID], target: Target, area: Area, range: u8, kind: SkillKind) -> Self {
        assert! (!status_ids.is_empty ());
        assert! (matches! (target, Target::This | Target::Ally | Target::Allies));

        Self { id, status_ids, target, area, range, kind }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn switch_status (&mut self) -> (ID, ID) {
        if let SkillKind::Toggled (status_idx_old) = self.kind {
            let status_id_old: ID = self.status_ids[status_idx_old];
            let status_idx_new: usize = (status_idx_old + 1) % self.status_ids.len ();
            let status_id_new: ID = self.status_ids[status_idx_new];

            self.kind = SkillKind::Toggled (status_idx_new);

            (status_id_old, status_id_new)
        } else {
            panic! ("Invalid skill kind {:?}", self.kind)
        }
    }

    pub fn start_cooldown (&mut self) {
        if let SkillKind::Timed (c, m) = self.kind {
            if c == 0 {
                self.kind = SkillKind::Timed (m, m);
            }
        } else {
            panic! ("Invalid skill kind {:?}", self.kind)
        }
    }

    pub fn is_timed (&self) -> bool {
        matches! (self.kind, SkillKind::Timed ( .. ))
    }

    pub fn is_passive (&self) -> bool {
        matches! (self.kind, SkillKind::Passive)
    }

    pub fn is_toggled (&self) -> bool {
        matches! (self.kind, SkillKind::Toggled ( .. ))
    }

    pub fn get_status_id (&self) -> ID {
        if let SkillKind::Toggled (status_idx) = self.kind {
            self.status_ids[status_idx]
        } else {
            self.status_ids[0]
        }
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
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        let status: Status = *scene.get_status (&self.get_status_id ());

        if self.is_timed () && self.get_duration () > 0 {
            None
        } else {
            status.try_yield_appliable (scene)
        }
    }

    fn get_target (&self) -> Target {
        self.target
    }
}

impl Timed for Skill {
    fn get_duration (&self) -> u16 {
        match self.kind {
            SkillKind::Timed (c, _) => c,
            SkillKind::Passive => DURATION_PERMANENT,
            SkillKind::Toggled ( .. ) => DURATION_PERMANENT,
        }
    }

    fn decrement_duration (&mut self) -> bool {
        match self.kind {
            SkillKind::Timed (c, m) => {
                if c == 0 {
                    true
                } else {
                    let duration: u16 = c.saturating_sub (1);

                    self.kind = SkillKind::Timed (duration, m);

                    false
                }
            }
            SkillKind::Passive => false,
            SkillKind::Toggled ( .. ) => false,
        }
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::tests::generate_scene;

    fn generate_skills () -> (Skill, Skill) {
        let scene = generate_scene ();
        let skill_0 = *scene.get_skill (&0);
        let skill_1 = *scene.get_skill (&1);

        (skill_0, skill_1)
    }

    #[test]
    fn skill_switch_status () {
        let scene = generate_scene ();
        let mut skill_2 = *scene.get_skill (&2);

        assert_eq! (skill_2.switch_status (), (0, 1));
        assert_eq! (skill_2.get_status_id (), 1);
        assert_eq! (skill_2.switch_status (), (1, 0));
        assert_eq! (skill_2.get_status_id (), 0);
    }

    #[test]
    fn skill_start_cooldown () {
        let (mut skill_0, _) = generate_skills ();

        // Test normal start
        skill_0.start_cooldown ();
        assert_eq! (skill_0.get_duration (), 2);
        //  Test interrupted start
        skill_0.kind = SkillKind::Timed (1, 2);
        skill_0.start_cooldown ();
        assert_eq! (skill_0.get_duration (), 1);
    }

    #[test]
    fn skill_decrement_duration () {
        let (mut skill_0, mut skill_1) = generate_skills ();

        // Test timed skill
        skill_0.start_cooldown ();
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
