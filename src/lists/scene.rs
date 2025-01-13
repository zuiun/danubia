use super::debug;
use crate::character::{FactionBuilder, Magic, Skill, UnitBuilder, Weapon};
use crate::common::ID;
use crate::dynamic::{Attribute, Effect, Modifier};
use crate::map::{City, Terrain, TileBuilder};

#[derive (Debug)]
pub struct Scene {
    modifiers: &'static [Modifier],
    effects: &'static [Effect],
    attributes: &'static [Attribute],
    terrains: &'static [Terrain],
    cities: &'static [City],
    weapons: &'static [Weapon],
    magics: &'static [Magic],
    skills: &'static [Skill],
    faction_builders: &'static [FactionBuilder],
    unit_builders: &'static [UnitBuilder],
    tile_builders: &'static [&'static [TileBuilder]],
}

impl Scene {
    pub fn new () -> Self {
        // TODO: Change this
        let modifiers: &[Modifier] = debug::MODIFIERS;
        let effects: &[Effect] = debug::EFFECTS;
        let attributes: &[Attribute] = debug::ATTRIBUTES;
        let terrains: &[Terrain] = debug::TERRAINS;
        let cities: &[City] = debug::CITIES;
        let weapons: &[Weapon] = debug::WEAPONS;
        let magics: &[Magic] = debug::MAGICS;
        let skills: &[Skill] = debug::SKILLS;
        let faction_builders: &[FactionBuilder] = debug::FACTION_BUILDERS;
        let unit_builders: &[UnitBuilder] = debug::UNIT_BUILDERS;
        let tile_builders: &[&[TileBuilder]] = debug::TILE_BUILDERS;

        Self { modifiers, effects, attributes, terrains, cities, weapons, magics, skills, faction_builders, unit_builders, tile_builders }
    }

    // pub fn debug () -> Self {
    //     let modifiers: &[Modifier] = debug::MODIFIERS;
    //     let effects: &[Effect] = debug::EFFECTS;
    //     let attributes: &[Attribute] = debug::ATTRIBUTES;
    //     let terrains: &[Terrain] = debug::TERRAINS;
    //     let cities: &[City] = debug::CITIES;
    //     let weapons: &[Weapon] = debug::WEAPONS;
    //     let magics: &[Magic] = debug::MAGICS;
    //     let skills: &[Skill] = debug::SKILLS;
    //     let faction_builders: &[FactionBuilder] = debug::FACTION_BUILDERS;
    //     let unit_builders: &[UnitBuilder] = debug::UNIT_BUILDERS;
    //     let tile_builders: &[&[TileBuilder]] = debug::TILE_BUILDERS;

    //     Self { modifiers, effects, attributes, terrains, cities, weapons, magics, skills, faction_builders, unit_builders, tile_builders }
    // }

    pub fn get_modifier (&self, id: &ID) -> &Modifier {
        assert! (*id < self.modifiers.len ());

        &self.modifiers[*id]
    }

    pub fn get_effect (&self, id: &ID) -> &Effect {
        assert! (*id < self.effects.len ());

        &self.effects[*id]
    }

    pub fn get_attribute (&self, id: &ID) -> &Attribute {
        assert! (*id < self.attributes.len ());

        &self.attributes[*id]
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

    pub fn get_tile_builders (&self) -> &[&[TileBuilder]] {
        self.tile_builders
    }
}

impl Default for Scene {
    fn default() -> Self {
        let modifiers: &[Modifier] = debug::MODIFIERS;
        let effects: &[Effect] = debug::EFFECTS;
        let attributes: &[Attribute] = debug::ATTRIBUTES;
        let terrains: &[Terrain] = debug::TERRAINS;
        let cities: &[City] = debug::CITIES;
        let weapons: &[Weapon] = debug::WEAPONS;
        let magics: &[Magic] = debug::MAGICS;
        let skills: &[Skill] = debug::SKILLS;
        let faction_builders: &[FactionBuilder] = debug::FACTION_BUILDERS;
        let unit_builders: &[UnitBuilder] = debug::UNIT_BUILDERS;
        let tile_builders: &[&[TileBuilder]] = debug::TILE_BUILDERS;

        Self { modifiers, effects, attributes, terrains, cities, weapons, magics, skills, faction_builders, unit_builders, tile_builders }
    }
}
