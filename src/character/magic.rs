use super::Tool;
use crate::common::{ID, Target};
use crate::dynamic::{Appliable, Applier, Status};
use crate::map::Area;
use crate::Lists;
use std::rc::Rc;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Element {
    Matter,
    Dark,
    Light,
    Length,
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct Magic {
    id: ID,
    status_id: ID,
    target: Target,
    area: Area,
    range: u8,
    cost: u16,
    element: Element,
}

impl Magic {
    pub const fn new (id: ID, status_id: ID, target: Target, area: Area, range: u8, cost: u16, element: Element) -> Self {
        assert! (matches! (target,
            Target::This
            | Target::Map
        ));

        Self { id, status_id, target, area, range, cost, element }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_status_id (&self) -> ID {
        self.status_id
    }

    pub fn get_cost (&self) -> u16 {
        self.cost
    }

    pub fn get_element (&self) -> Element {
        self.element
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
        let status: Status = *lists.get_status (&self.status_id);

        status.try_yield_appliable (lists)
    }

    fn get_target (&self) -> Target {
        self.target
    }
}
