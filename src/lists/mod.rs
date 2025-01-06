use crate::character::{FactionBuilder, Magic, Skill, UnitBuilder, Weapon};
use crate::common::ID;
use crate::dynamic::{Effect, ModifierBuilder, Status};
use crate::map::{City, Terrain};

pub mod debug;
pub mod game;
pub mod information;

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
    #[allow (clippy::new_without_default)]
    pub fn new () -> Self {
        let modifier_builders = Box::new (game::MODIFIER_BUILDERS);
        let effects = Box::new (game::EFFECTS);
        let statuses = Box::new (game::STATUSES);
        let terrains = Box::new (game::TERRAINS);
        let cities = Box::new (game::CITIES);
        let weapons = Box::new (game::WEAPONS);
        let magics = Box::new (game::MAGICS);
        let skills = Box::new (game::SKILLS);
        let faction_builders = Box::new (game::FACTION_BUILDERS);
        let unit_builders = Box::new (game::UNIT_BUILDERS);

        Self { modifier_builders, effects, statuses, terrains, cities, weapons, magics, skills, faction_builders, unit_builders }
    }

    pub fn debug () -> Self {
        let modifier_builders = Box::new (debug::MODIFIER_BUILDERS);
        let effects = Box::new (debug::EFFECTS);
        let statuses = Box::new (debug::STATUSES);
        let terrains = Box::new (debug::TERRAINS);
        let cities = Box::new (debug::CITIES);
        let weapons = Box::new (debug::WEAPONS);
        let magics = Box::new (debug::MAGICS);
        let skills = Box::new (debug::SKILLS);
        let faction_builders = Box::new (debug::FACTION_BUILDERS);
        let unit_builders = Box::new (debug::UNIT_BUILDERS);

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

    pub fn magics_iter (&self) -> impl Iterator<Item = &Magic> {
        self.magics.iter ()
    }

    pub fn get_skill (&self, id: &ID) -> &Skill {
        assert! (*id < self.skills.len ());

        &self.skills[*id]
    }

    // pub fn skills_iter (&self) -> impl Iterator<Item = &Skill> {
    //     self.skills.iter ()
    // }

    pub fn get_faction_builder (&self, id: &ID) -> &FactionBuilder {
        assert! (*id < self.faction_builders.len ());

        &self.faction_builders[*id]
    }

    pub fn faction_builders_iter (&self) -> impl Iterator<Item = &FactionBuilder> {
        self.faction_builders.iter ()
    }

    pub fn get_unit_builder (&self, id: &ID) -> &UnitBuilder {
        assert! (*id < self.unit_builders.len ());

        &self.unit_builders[*id]
    }

    pub fn unit_builders_iter (&self) -> impl Iterator<Item = &UnitBuilder> {
        self.unit_builders.iter ()
    }
}
