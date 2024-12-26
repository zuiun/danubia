pub mod common;
pub mod event;
pub mod map;
pub mod character;

use std::rc::Rc;
use common::{Information, ID, Modifier, Status};
use map::{City, Terrain};
use character::{Action, Faction, Magic, Skill, Unit, Weapon};
use super::lists;

/*
 * Calculated from build.rs
 * Unit MOV is an index into the table
 * Attack (* 1.0): 21 delay at 0, 20 delay at 1, 2 delay at 77, and 1 delay at 100
 * Magic/skill (* 1.4): 29 delay at 0, 28 delay at 1, 2 delay at 77, and 1 delay at 100
 * Wait (* 0.67): 14 delay at 0, 13 delay at 1, 2 delay at 54, and 1 delay at 77
 */
const DELAYS: [u8; 101] = [21, 20, 19, 19, 18, 18, 17, 17, 16, 16, 15, 15, 14, 14, 14, 13, 13, 13, 12, 12, 11, 11, 11, 11, 10, 10, 10, 9, 9, 9, 9, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1];
const DELAY_APPLIER: f32 = 1.4;
const DELAY_WAIT: f32 = 0.67;

// TODO: Anything that has an ID also has an Information mapped to it

#[derive (Debug)]
pub struct Lists {
    delays: [u8; 101],
    modifiers: Box<[Modifier]>,
    statuses: Box<[Status]>,
    terrains: Box<[Terrain]>,
    cities: Box<[City]>,
    weapons: Box<[Weapon]>,
    magics: Box<[Magic]>,
    skills: Box<[Skill]>,
    factions: Box<[Faction]>,
    units: Box<[Unit]>
}

#[derive (Debug)]
pub struct Game {
    lists: Lists
}

impl Lists {
    pub fn new () -> Self {
        let delays: [u8; 101] = DELAYS;
        let modifiers: Box<[Modifier]> = Box::new (lists::game::MODIFIERS);
        let statuses: Box<[Status]> = Box::new (lists::game::STATUSES);
        let terrains: Box<[Terrain]> = Box::new (lists::game::TERRAINS);
        let cities: Box<[City]> = Box::new (lists::game::CITIES);
        let weapons: Box<[Weapon]> = Box::new (lists::game::WEAPONS);
        let magics: Box<[Magic]> = Box::new (lists::game::MAGICS);
        let skills: Box<[Skill]> = Box::new (lists::game::SKILLS);
        let factions: Box<[Faction]> = Box::new (lists::game::FACTIONS);
        let units: Box<[Unit]> = Box::new (lists::game::UNITS);

        Self { modifiers, statuses, delays, terrains, cities, weapons, magics, skills, factions, units }
    }

    pub fn debug () -> Self {
        let delays: [u8; 101] = DELAYS;
        let modifiers: Box<[Modifier]> = Box::new (lists::debug::MODIFIERS);
        let statuses: Box<[Status]> = Box::new (lists::debug::STATUSES);
        let terrains: Box<[Terrain]> = Box::new (lists::debug::TERRAINS);
        let cities: Box<[City]> = Box::new (lists::debug::CITIES);
        let weapons: Box<[Weapon]> = Box::new (lists::debug::WEAPONS);
        let magics: Box<[Magic]> = Box::new (lists::debug::MAGICS);
        let skills: Box<[Skill]> = Box::new (lists::debug::SKILLS);
        let factions: Box<[Faction]> = Box::new (lists::debug::FACTIONS);
        let units: Box<[Unit]> = Box::new (lists::debug::UNITS);

        Self { modifiers, statuses, delays, terrains, cities, weapons, magics, skills, factions, units }
    }

    pub fn get_delay (&self, mov: u16, action: Action) -> u8 {
        assert! ((mov as usize) < self.delays.len ());

        let delay: u8 = self.delays[mov as usize];

        match action {
            Action::Attack (_) => delay,
            Action::Wait => ((delay as f32) * DELAY_WAIT) as u8,
            _ => ((delay as f32) * DELAY_APPLIER) as u8
        }
    }

    pub fn get_modifier (&self, id: &ID) -> &Modifier {
        assert! (*id < self.modifiers.len ());

        &self.modifiers[*id]
    }

    pub fn get_status (&self, id: &ID) -> &Status {
        assert! (*id < self.statuses.len ());

        &self.statuses[*id]
    }

    pub fn get_terrain (&self, id: &ID) -> &Terrain {
        assert! (*id < self.terrains.len ());

        &self.terrains[*id]
    }

    pub fn get_city (&self, id: &ID) -> &City {
        assert! (*id < self.cities.len ());

        &self.cities[*id]
    }

    pub fn get_weapon (&self, id: &ID) -> &Weapon {
        assert! (*id < self.weapons.len ());

        &self.weapons[*id]
    }

    pub fn get_magic (&self, id: &ID) -> &Magic {
        assert! (*id < self.magics.len ());

        &self.magics[*id]
    }

    pub fn get_skill (&self, id: &ID) -> &Skill {
        assert! (*id < self.skills.len ());

        &self.skills[*id]
    }

    pub fn get_faction (&self, id: &ID) -> &Faction {
        assert! (*id < self.factions.len ());

        &self.factions[*id]
    }

    pub fn get_unit (&self, id: &ID) -> &Unit {
        assert! (*id < self.units.len ());

        &self.units[*id]
    }
}

impl Game {
    pub fn new () -> Self {
        let lists: Lists = Lists::new ();

        Self { lists }
    }

    pub fn debug () -> Self {
        let lists: Lists = Lists::debug ();

        Self { lists }
    }

    pub fn update () -> () {
        //
    }
}

pub mod tests {
    use super::*;

    pub fn generate_lists () -> Rc<Lists> {
        Rc::new (Lists::debug ())
    }
}
