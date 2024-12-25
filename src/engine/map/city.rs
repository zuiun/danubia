use std::{cell::RefCell, cmp, rc::Rc};
use crate::engine::event::{Event, Subject, Observer, Result, RESULT_NOTIFICATION};

const RECOVER_MANPOWER_DIVIDEND: u16 = 10;

#[derive (Debug)]
pub struct City {
    population: u16, // (thousands)
    factories: u16,
    farms: u16,
    draw_count: u16,
    stockpile: (u16, u16),
    observers: Vec<Rc<RefCell<dyn Observer>>>
}

impl City {
    pub const fn new (population: u16, factories: u16, farms: u16) -> Self {
        let draw_count: u16 = 0;
        let stockpile: (u16, u16) = (0, 0);
        let observers: Vec<Rc<RefCell<dyn Observer>>> = Vec::new ();

        Self { population, factories, farms, draw_count, stockpile, observers }
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

impl Subject for City {
    fn add_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        let observer: Rc<RefCell<dyn Observer>> = Rc::clone (&observer);

        self.observers.push (observer);
    }

    fn remove_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        unimplemented! ()
    }

    async fn notify (&self, event: Event) -> Result {
        for observer in self.observers.iter () {
            observer.borrow_mut ().update (event);
        }

        RESULT_NOTIFICATION
    }
}

impl Observer for City {
    fn update (&mut self, event: Event) -> () {
        todo!()
    }
}


