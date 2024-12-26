use std::{matches, rc::Rc};
use crate::engine::Lists;
use crate::engine::common::{Area, Capacity, ID, Modifiable, Modifier, Status, Target, Timed};

const SKILL_1: usize = 0;
const SKILL_2: usize = 1;
const SKILL_ACTIVE: usize = 2;
const TOGGLE_OFF: u16 = 0;
const TOGGLE_ON: u16 = 1;

pub trait Applier {
    fn prepare (&mut self, lists: Rc<Lists>) -> Option<Modifier>;
    fn get_target (&self) -> Target;
    fn get_area (&self) -> Area;
    fn get_range (&self) -> u8;
}

#[derive (Debug)]
pub struct Magic {
    status_id: ID,
    target: Target,
    area: Area,
    range: u8
}

impl Magic {
    pub const fn new (status_id: ID, target: Target, area: Area, range: u8) -> Self {
        Self { status_id, target, area, range }
    }

    pub fn get_status_id (&self) -> ID {
        self.status_id
    }
}

impl Applier for Magic {
    fn prepare (&mut self, lists: Rc<Lists>) -> Option<Modifier> {
        let status: &Status = lists.get_status (&self.status_id);
        let modifier_id: ID = status.get_modifier_id ();
        let modifier: &Modifier = lists.get_modifier (&modifier_id);

        Some (modifier.clone ())
    }

    fn get_target (&self) -> Target {
        self.target
    }

    fn get_area (&self) -> Area {
        self.area
    }

    fn get_range (&self) -> u8 {
        self.range
    }
}

#[derive (Debug)]
pub struct Skill {
    status_ids: [ID; 3],
    target: Target,
    area: Area,
    range: u8,
    cooldown: Capacity // constant (passive, toggle one, toggle two), quantity (duration, maximum)
}

impl Skill {
    pub const fn new (status_ids: [ID; 3], target: Target, area: Area, range: u8, duration: Capacity) -> Self {
        assert! (matches! (target, Target::Ally { .. }) || matches! (target, Target::All (true)));

        Self { status_ids, target, area, range, cooldown: duration }
    }

    pub fn get_status_ids (&self) -> &[ID; 3] {
        &self.status_ids
    }
}

impl Applier for Skill {
    fn prepare (&mut self, lists: Rc<Lists>) -> Option<Modifier> {
        self.cooldown = match self.cooldown {
            Capacity::Constant (TOGGLE_OFF, t1, t2) => {
                let mut toggle_first: u16 = TOGGLE_OFF;
                let mut toggle_second: u16 = TOGGLE_OFF;

                self.status_ids[SKILL_ACTIVE] = if self.status_ids[SKILL_ACTIVE] == self.status_ids[SKILL_1] {
                    assert_eq! (t1, TOGGLE_ON);
                    assert_eq! (t2, TOGGLE_OFF);

                    toggle_second = TOGGLE_ON;

                    self.status_ids[SKILL_2]
                } else {
                    assert_eq! (t1, TOGGLE_OFF);
                    assert_eq! (t2, TOGGLE_ON);

                    toggle_first = TOGGLE_ON;

                    self.status_ids[SKILL_1]
                };

                Capacity::Constant (0, toggle_first, toggle_second)
            }
            Capacity::Constant (_, TOGGLE_OFF, TOGGLE_OFF) => Capacity::Constant (TOGGLE_ON, TOGGLE_OFF, TOGGLE_OFF),
            Capacity::Quantity (c, m) => {
                if c > 0 {
                    return None
                } else {
                    Capacity::Quantity (m, m)
                }
            }
            _ => panic! ("Invalid cooldown {:?}", self.cooldown)
        };

        let status: &Status = lists.get_status (&self.status_ids[SKILL_ACTIVE]);
        let modifier_id: ID = status.get_modifier_id ();
        let modifier: &Modifier = lists.get_modifier (&modifier_id);

        Some (modifier.clone ())
    }

    fn get_target (&self) -> Target {
        self.target
    }

    fn get_area (&self) -> Area {
        self.area
    }

    fn get_range (&self) -> u8 {
        self.range
    }
}

impl Timed for Skill {
    fn get_duration (&self) -> u16 {
        match self.cooldown {
            Capacity::Constant (_, _, _) => u16::MAX,
            Capacity::Quantity (d, _) => d
        }
    }

    fn dec_duration (&mut self) -> bool {
        match self.cooldown {
            Capacity::Constant (_, _, _) => false,
            Capacity::Quantity (d, m) => {
                let duration: u16 = d.checked_sub (1).unwrap_or (0);

                self.cooldown = Capacity::Quantity (duration, m);

                duration == 0
            }
        }
    }
}

#[cfg (test)]
mod tests {
    // TODO
}
