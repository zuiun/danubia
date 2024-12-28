use std::rc::Rc;
use crate::engine::Lists;
use crate::engine::common::{Area, ID, Target};
use crate::engine::dynamic::{Appliable, Applier, Change, Effect, Modifier, Status};

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Magic {
    status_id: ID,
    target: Target,
    area: Area,
    range: u8,
}

impl Magic {
    pub const fn new (status_id: ID, target: Target, area: Area, range: u8) -> Self {
        Self { status_id, target, area, range }
    }

    pub fn get_status_id (&self) -> ID {
        self.status_id
    }

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

    fn get_target (&self) -> Option<Target> {
        Some (self.target)
    }
}
