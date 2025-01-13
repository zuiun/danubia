use super::Tool;
use crate::common::{ID, Target};
use crate::dynamic::{Appliable, AppliableKind, Applier};
use crate::map::Area;
use crate::Scene;
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
    appliable: AppliableKind,
    target: Target,
    area: Area,
    range: u8,
    cost: u16,
    element: Element,
}

impl Magic {
    pub const fn new (id: ID, appliable: AppliableKind, target: Target, area: Area, range: u8, cost: u16, element: Element) -> Self {
        assert! (matches! (target, Target::This | Target::Map));

        Self { id, appliable, target, area, range, cost, element }
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_appliable (&self) -> AppliableKind {
        self.appliable
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
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        Some (self.get_appliable ().appliable (scene))
    }

    fn get_target (&self) -> Target {
        self.target
    }
}
