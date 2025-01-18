use crate::common::ID;

pub const WORKERS_FACTORY: f32 = 4.0;
pub const WORKERS_FARM: f32 = 1.5;
const RECOVER_MANPOWER_MODIFIER: f32 = 5.0;
const RECOVER_EQUIPMENT_MODIFIER: f32 = 2.5;
const MODIFIER_MINIMUM: f32 = 0.67;

#[derive (Debug)]
pub struct City {
    population: u16, // (thousands)
    factories: u16,
    farms: u16,
    recruit_id: Option<ID>,
}

impl City {
    pub const fn new (population: u16, factories: u16, farms: u16, recruit_id: Option<ID>) -> Self {
        assert! (population > 0);
        assert! (factories > 0);
        assert! (farms > 0);

        Self { population, factories, farms, recruit_id }
    }

    pub const fn get_workers (&self) -> u16 {
        let workers_factory: f32 = (self.factories as f32) * WORKERS_FACTORY;
        let workers_farm: f32 = (self.farms as f32) * WORKERS_FARM;
        let workers: u16 = (workers_factory + workers_farm) as u16;

        workers
    }

    pub fn get_manpower (&self) -> u16 {
        let factories: f32 = self.factories as f32;
        let population: f32 = self.population as f32;
        let farms: f32 = self.farms as f32;
        let workers: f32 = self.get_workers () as f32;
        let modifier: f32 = f32::max ((farms / RECOVER_MANPOWER_MODIFIER) / factories, MODIFIER_MINIMUM);
        let recruitable: f32 = f32::max (population - workers, 1.0);
        let manpower: u16 = (recruitable * modifier).ceil () as u16;

        manpower
    }

    pub fn get_equipment (&self) -> u16 {
        let factories: f32 = self.factories as f32;
        let farms: f32 = self.farms as f32;
        let modifier: f32 = f32::max ((factories - RECOVER_EQUIPMENT_MODIFIER) / farms, MODIFIER_MINIMUM);
        let production: f32 = (factories + modifier).sqrt ();
        let equipment: u16 = (production * modifier).ceil () as u16;

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

    pub fn get_recruit_id (&self) -> Option<ID> {
        self.recruit_id
    }
}

#[cfg (test)]
mod tests {
    // use super::*;
    // use crate::common::Information;
    // use crate::debug;
    // use crate::lists::information;
    use crate::tests::generate_scene;

    #[test]
    fn city_get_manpower () {
        let scene = generate_scene ();

        for i in 0 ..= 3 {
            assert! (scene.get_city (&i).get_manpower () > 0);
        }

        let manpower_0: u16 = scene.get_city (&0).get_manpower ();
        let manpower_1: u16 = scene.get_city (&1).get_manpower ();
        let manpower_2: u16 = scene.get_city (&2).get_manpower ();
        let manpower_3: u16 = scene.get_city (&3).get_manpower ();

        println! ("{}", manpower_0);
        println! ("{}", manpower_1);
        println! ("{}", manpower_2);
        println! ("{}", manpower_3);
        // assert! (false);
    }

    #[test]
    fn city_get_equipment () {
        let scene = generate_scene ();
        let equipment_0: u16 = scene.get_city (&0).get_equipment ();
        let equipment_1: u16 = scene.get_city (&1).get_equipment ();
        let equipment_2: u16 = scene.get_city (&2).get_equipment ();
        let equipment_3: u16 = scene.get_city (&3).get_equipment ();

        for i in 0 ..= 3 {
            assert! (scene.get_city (&i).get_manpower () > 0);
        }

        println! ("{}", equipment_0);
        println! ("{}", equipment_1);
        println! ("{}", equipment_2);
        println! ("{}", equipment_3);
        // assert! (false);
    }

    // #[test]
    // fn cities_balance () {
    //     for i in 4 .. debug::CITIES.len () {
    //         let city: &City = &debug::CITIES[i];
    //         let information: &Information = &information::CITIES[i];
    //         let name: &str = information.get_name ();
    //         let population: u16 = city.get_population ();
    //         let workers: u16 = city.get_workers ();
    //         let manpower: u16 = city.get_manpower ();
    //         let equipment: u16 = city.get_equipment ();

    //         println! ("{}: {} ? {} -> {}, {}", name, workers, population, manpower, equipment);
    //         assert! (workers < population);
    //     }

    //     // assert! (false);
    // }
}
