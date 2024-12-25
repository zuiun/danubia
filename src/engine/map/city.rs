use std::{cell::RefCell, cmp, rc::Rc};
use crate::engine::common::{ID, ID_UNINITIALISED};
use crate::engine::event::{Event, Subject, Observer, Response, RESPONSE_NOTIFICATION};

const RECOVER_MANPOWER_DIVIDEND: u16 = 10;

#[derive (Debug)]
pub struct City {
    population: u16, // (thousands)
    factories: u16,
    farms: u16,
    draw_count: u16,
    stockpile: (u16, u16),
    observer_id: ID
}

impl City {
    pub const fn new (population: u16, factories: u16, farms: u16) -> Self {
        let draw_count: u16 = 0;
        let stockpile: (u16, u16) = (0, 0);
        let observer_id: ID = ID_UNINITIALISED;

        Self { population, factories, farms, draw_count, stockpile, observer_id }
    }

    pub fn get_manpower (&self) -> u16 {
        let modifier: f32 = (self.farms as f32) / (self.factories as f32);
        let manpower: u16 = cmp::max ((((self.population / RECOVER_MANPOWER_DIVIDEND) as f32) * modifier) as u16, 1);

        manpower
    }

    pub fn get_equipment (&self) -> u16 {
        let equipment: u16 = self.factories;

        equipment
    }

    pub fn draw_supplies (&mut self) -> (u16, u16) {
        let manpower: u16 = self.get_manpower ();
        let equipment: u16 = self.get_equipment ();

        self.draw_count += 1;

        if self.draw_count % RECOVER_MANPOWER_DIVIDEND == 0 {
            self.population -= 1;
        }


        (manpower, equipment)
    }

    pub fn get_population (&self) -> u16 {
        self.population
    }

    pub fn get_factories (&self) -> u16 {
        self.factories
    }

    pub fn get_farms (&self) -> u16 {
        self.farms
    }
}

impl Observer for City {
    fn subscribe (&mut self, event_id: ID) -> ID {
        todo! ()
    }

    fn unsubscribe (&mut self, event_id: ID) -> ID {
        todo! ()   
    }

    fn update (&mut self, event_id: ID) -> () {
        todo! ()
    }

    fn get_observer_id (&self) -> Option<ID> {
        if self.observer_id == ID_UNINITIALISED {
            None
        } else {
            Some (self.observer_id)
        }
    }
}

impl Subject for City {
    async fn notify (&self, event: Event) -> Response {
        RESPONSE_NOTIFICATION
    }
}
