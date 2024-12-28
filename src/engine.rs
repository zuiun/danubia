pub mod common;
pub mod duplicate_map;
pub mod dynamic;
pub mod event;
pub mod map;
pub mod character;

mod information;
pub use information::Information;

use std::rc::Rc;
use common::ID;
use map::{City, Terrain};
use dynamic::{Effect, ModifierBuilder, Status};
use character::{Action, Faction, Magic, Skill, Unit, Weapon};
use super::lists;

// TODO: Anything that has an ID also has an Information mapped to it

#[derive (Debug)]
pub struct Lists {
    modifier_builders: Box<[ModifierBuilder]>,
    effects: Box<[Effect]>,
    statuses: Box<[Status]>,
    terrains: Box<[Terrain]>,
    cities: Box<[City]>,
    weapons: Box<[Weapon]>,
    magics: Box<[Magic]>,
    skills: Box<[Skill]>,
    factions: Box<[Faction]>,
    units: Box<[Unit]>,
}

#[derive (Debug)]
pub struct Game {
    lists: Lists,
}

impl Lists {
    pub fn new () -> Self {
        let modifier_builders: Box<[ModifierBuilder]> = Box::new (lists::game::MODIFIER_BUILDERS);
        let effects: Box<[Effect]> = Box::new (lists::game::EFFECTS);
        let statuses: Box<[Status]> = Box::new (lists::game::STATUSES);
        let terrains: Box<[Terrain]> = Box::new (lists::game::TERRAINS);
        let cities: Box<[City]> = Box::new (lists::game::CITIES);
        let weapons: Box<[Weapon]> = Box::new (lists::game::WEAPONS);
        let magics: Box<[Magic]> = Box::new (lists::game::MAGICS);
        let skills: Box<[Skill]> = Box::new (lists::game::SKILLS);
        let factions: Box<[Faction]> = Box::new (lists::game::FACTIONS);
        let units: Box<[Unit]> = Box::new (lists::game::UNITS);

        Self { modifier_builders, effects, statuses, terrains, cities, weapons, magics, skills, factions, units }
    }

    pub fn debug () -> Self {
        let modifier_builders: Box<[ModifierBuilder]> = Box::new (lists::debug::MODIFIER_BUILDERS);
        let effects: Box<[Effect]> = Box::new (lists::debug::EFFECTS);
        let statuses: Box<[Status]> = Box::new (lists::debug::STATUSES);
        let terrains: Box<[Terrain]> = Box::new (lists::debug::TERRAINS);
        let cities: Box<[City]> = Box::new (lists::debug::CITIES);
        let weapons: Box<[Weapon]> = Box::new (lists::debug::WEAPONS);
        let magics: Box<[Magic]> = Box::new (lists::debug::MAGICS);
        let skills: Box<[Skill]> = Box::new (lists::debug::SKILLS);
        let factions: Box<[Faction]> = Box::new (lists::debug::FACTIONS);
        let units: Box<[Unit]> = Box::new (lists::debug::UNITS);

        Self { modifier_builders, effects, statuses, terrains, cities, weapons, magics, skills, factions, units }
    }

    pub fn get_modifier_builder (&self, id: &ID) -> &ModifierBuilder {
        assert! (*id < self.modifier_builders.len ());

        &self.modifier_builders[*id]
    }

    pub fn get_effect (&self, id: &ID) -> &Effect {
        assert! (*id < self.effects.len ());

        &self.effects[*id]
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
