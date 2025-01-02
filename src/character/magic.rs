use std::rc::Rc;
use crate::Lists;
use crate::common::{ID, Target};
use crate::dynamic::{Appliable, Applier, Status};
use crate::map::Area;
use super::Tool;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Magic {
    status_id: ID,
    target: Target,
    area: Area,
    range: u8,
    cost: u16,
}

impl Magic {
    pub const fn new (status_id: ID, target: Target, area: Area, range: u8, cost: u16) -> Self {
        Self { status_id, target, area, range, cost }
    }

    pub fn get_status_id (&self) -> ID {
        self.status_id
    }

    pub fn get_cost (&self) -> u16 {
        self.cost
    }
}

impl Tool for Magic {
    fn get_area (&self) -> Area {
        self.area
    }

    fn get_range (&self) -> u8 {
        self.range
    }
}

impl Applier for Magic {
    fn try_yield_appliable (&self, lists: Rc<Lists>) -> Option<Box<dyn Appliable>> {
        let status: Status = lists.get_status (&self.status_id).clone ();

        status.try_yield_appliable (lists)
    }

    fn get_target (&self) -> Target {
        self.target
    }
}
