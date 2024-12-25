use std::{matches, rc::Rc};
use crate::engine::Lists;
use crate::engine::common::{Area, Capacity, ID, Modifiable, Modifier, Status, Target, Timed};

const TOGGLE_1: usize = 0;
const TOGGLE_2: usize = 1;
const TOGGLE_ACTIVE: usize = 2;

pub trait Applier {
    fn prepare (&mut self, lists: Rc<Lists>) -> Modifier;
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
    fn prepare (&mut self, lists: Rc<Lists>) -> Modifier {
        let status: &Status = lists.get_status (&self.status_id);
        let modifier_id: ID = status.get_modifier_id ();
        let modifier: &Modifier = lists.get_modifier (&modifier_id);

        modifier.clone ()
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
    duration: Capacity // constant (passive, toggle one, toggle two), quantity (duration, maximum)
}

impl Skill {
    pub const fn new (status_ids: [ID; 3], target: Target, area: Area, range: u8, duration: Capacity) -> Self {
        assert! (matches! (target, Target::Ally { .. }) || matches! (target, Target::All (true)));

        Self { status_ids, target, area, range, duration }
    }

    pub fn get_status_ids (&self) -> &[ID; 3] {
        &self.status_ids
    }
}

impl Applier for Skill {
    fn prepare (&mut self, lists: Rc<Lists>) -> Modifier {
        if let Capacity::Constant (0, t1, t2) = self.duration {
            self.status_ids[TOGGLE_ACTIVE] = if self.status_ids[TOGGLE_ACTIVE] == self.status_ids[TOGGLE_1] {
                assert! (t1 > 0);

                self.duration = Capacity::Constant (0, 0, 1);

                self.status_ids[TOGGLE_2]
            } else {
                assert! (t2 > 0);

                self.duration = Capacity::Constant (0, 1, 0);

                self.status_ids[TOGGLE_1]
            };
        }

        let status: &Status = lists.get_status (&self.status_ids[TOGGLE_ACTIVE]);
        let modifier_id: ID = status.get_modifier_id ();
        let modifier: &Modifier = lists.get_modifier (&modifier_id);

        modifier.clone ()
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
        match self.duration {
            Capacity::Constant (_, _, _) => u16::MAX,
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
