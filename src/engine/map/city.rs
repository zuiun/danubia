use std::{cell::RefCell, cmp, rc::Rc};
use crate::engine::common::{ID, ID_UNINITIALISED};
use crate::engine::event::{Event, Subject, Observer, Response, RESPONSE_NOTIFICATION};

const RECOVER_MANPOWER_DIVIDEND: u16 = 10;

#[derive (Debug)]
pub struct City {
    population: u16, // (thousands)
    factories: u16,
    farms: u16,
    observer_id: ID
}

impl City {
    pub const fn new (population: u16, factories: u16, farms: u16) -> Self {
        assert! (population > 0);
        assert! (factories > 0);
        assert! (farms > 0);

        let observer_id: ID = ID_UNINITIALISED;

        Self { population, factories, farms, observer_id }
    }

    const fn get_workers (&self) -> u16 {
        self.factories + self.farms
    }

    pub fn get_manpower (&self) -> u16 {
        let workers: u16 = self.get_workers ();
        let modifier: f32 = (self.farms as f32) / (self.factories as f32);
        let recruitable: f32 = ((self.population / RECOVER_MANPOWER_DIVIDEND) as f32) * modifier;
        let manpower: u16 = (recruitable as u16).checked_sub (workers).unwrap_or (1);

        manpower
    }

    pub fn get_equipment (&self) -> u16 {
        let equipment: u16 = ((self.factories - 1) as f32).sqrt () as u16;

        equipment
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
    async fn update (&mut self, event: Event) -> Response {
        todo! ()
    }

    fn get_observer_id (&self) -> Option<ID> {
        if self.observer_id == ID_UNINITIALISED {
            None
        } else {
            Some (self.observer_id)
        }
    }

    fn set_observer_id (&mut self, observer_id: ID) -> () {
        self.observer_id = observer_id;
    }
}

impl Subject for City {
    async fn notify (&self, event: Event) -> Response {
        RESPONSE_NOTIFICATION
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use crate::engine::{Lists, tests::generate_lists};

    #[test]
    fn city_get_manpower () {
        let lists: Rc<Lists> = generate_lists ();
        let manpower_0: u16 = lists.get_city (&0).get_manpower ();
        let manpower_1: u16 = lists.get_city (&1).get_manpower ();
        let manpower_2: u16 = lists.get_city (&2).get_manpower ();
        let manpower_3: u16 = lists.get_city (&3).get_manpower ();

        assert! (manpower_0 > manpower_1);
        assert! (manpower_0 < manpower_2);
        // assert! (manpower_0 > manpower_3);
        assert! (manpower_1 < manpower_2);
        assert! (manpower_1 < manpower_3);
        assert! (manpower_2 > manpower_3);

        println! ("{}", manpower_0);
        println! ("{}", manpower_1);
        println! ("{}", manpower_2);
        println! ("{}", manpower_3);
        assert! (false);
    }

    #[test]
    fn city_get_equipment () {
        let lists: Rc<Lists> = generate_lists ();
        let city: City = City::new (10, 1, 1);
        let equipment_0: u16 = lists.get_city (&0).get_equipment ();
        let equipment_1: u16 = lists.get_city (&1).get_equipment ();
        let equipment_2: u16 = lists.get_city (&2).get_equipment ();
        let equipment_3: u16 = lists.get_city (&3).get_equipment ();

        assert_eq! (equipment_0, 0);
        // assert_eq! (equipment_1, 1);
        assert_eq! (equipment_2, 0);
        // assert_eq! (equipment_3, 1);

        println! ("{}", equipment_0);
        println! ("{}", equipment_1);
        println! ("{}", equipment_2);
        println! ("{}", equipment_3);
        assert! (false);
    }
}
