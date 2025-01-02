pub mod common;
pub mod join_map;
pub mod dynamic;
pub mod event;
pub mod map;
pub mod character;

mod lists;

use std::rc::Rc;
use common::{ID, ID_UNINITIALISED};
use character::{FactionBuilder, Magic, Skill, UnitBuilder, Weapon};
use dynamic::{Effect, ModifierBuilder, Status};
use event::Handler;
use map::{City, Terrain};

// TODO: Anything that has an ID also has an Information mapped to it

#[derive (Debug)]
pub struct Game {
    lists: Lists,
    handler: Handler,
}

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
    faction_builders: Box<[FactionBuilder]>,
    unit_builders: Box<[UnitBuilder]>,
}

impl Lists {
    pub fn new () -> Self {
        let modifier_builders = Box::new (lists::game::MODIFIER_BUILDERS);
        let effects = Box::new (lists::game::EFFECTS);
        let statuses = Box::new (lists::game::STATUSES);
        let terrains = Box::new (lists::game::TERRAINS);
        let cities = Box::new (lists::game::CITIES);
        let weapons = Box::new (lists::game::WEAPONS);
        let magics = Box::new (lists::game::MAGICS);
        let skills = Box::new (lists::game::SKILLS);
        let faction_builders = Box::new (lists::game::FACTION_BUILDERS);
        let unit_builders = Box::new (lists::game::UNIT_BUILDERS);

        Self { modifier_builders, effects, statuses, terrains, cities, weapons, magics, skills, faction_builders, unit_builders }
    }

    pub fn debug () -> Self {
        let modifier_builders = Box::new (lists::debug::MODIFIER_BUILDERS);
        let effects = Box::new (lists::debug::EFFECTS);
        let statuses = Box::new (lists::debug::STATUSES);
        let terrains = Box::new (lists::debug::TERRAINS);
        let cities = Box::new (lists::debug::CITIES);
        let weapons = Box::new (lists::debug::WEAPONS);
        let magics = Box::new (lists::debug::MAGICS);
        let skills = Box::new (lists::debug::SKILLS);
        let faction_builders = Box::new (lists::debug::FACTION_BUILDERS);
        let unit_builders = Box::new (lists::debug::UNIT_BUILDERS);

        Self { modifier_builders, effects, statuses, terrains, cities, weapons, magics, skills, faction_builders, unit_builders }
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

    pub fn get_faction_builder (&self, id: &ID) -> &FactionBuilder {
        assert! (*id < self.faction_builders.len ());

        &self.faction_builders[*id]
    }

    pub fn get_unit_builder (&self, id: &ID) -> &UnitBuilder {
        assert! (*id < self.unit_builders.len ());

        &self.unit_builders[*id]
    }
}

impl Game {
    pub fn new () -> Self {
        let lists = Lists::new ();
        let handler = Handler::new ();

        Self { lists, handler }
    }

    pub fn debug () -> Self {
        let lists = Lists::debug ();
        let handler = Handler::new ();

        Self { lists, handler }
    }

    pub fn update () -> () {
        todo! ()
    }
}

pub mod tests {
    use super::*;

    pub fn generate_lists () -> Rc<Lists> {
        Rc::new (Lists::debug ())
    }
}
