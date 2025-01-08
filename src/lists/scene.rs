use super::{debug, game};
use crate::character::{FactionBuilder, Magic, Skill, UnitBuilder, Weapon};
use crate::common::ID;
use crate::dynamic::{Effect, ModifierBuilder, Status};
use crate::map::{City, Terrain, TileBuilder};

#[derive (Debug)]
pub struct Scene {
    modifier_builders: &'static [ModifierBuilder],
    effects: &'static [Effect],
    statuses: &'static [Status],
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
    #[allow (clippy::new_without_default)]
    pub fn new () -> Self {
        let modifier_builders: &[ModifierBuilder] = game::MODIFIER_BUILDERS;
        let effects: &[Effect] = game::EFFECTS;
        let statuses: &[Status] = game::STATUSES;
        let terrains: &[Terrain] = game::TERRAINS;
        let cities: &[City] = game::CITIES;
        let weapons: &[Weapon] = game::WEAPONS;
        let magics: &[Magic] = game::MAGICS;
        let skills: &[Skill] = game::SKILLS;
        let faction_builders: &[FactionBuilder] = game::FACTION_BUILDERS;
        let unit_builders: &[UnitBuilder] = game::UNIT_BUILDERS;
        let tile_builders: &[&[TileBuilder]] = game::TILE_BUILDERS;

        Self { modifier_builders, effects, statuses, terrains, cities, weapons, magics, skills, faction_builders, unit_builders, tile_builders }
    }

    pub fn debug () -> Self {
        let modifier_builders: &[ModifierBuilder] = debug::MODIFIER_BUILDERS;
        let effects: &[Effect] = debug::EFFECTS;
        let statuses: &[Status] = debug::STATUSES;
        let terrains: &[Terrain] = debug::TERRAINS;
        let cities: &[City] = debug::CITIES;
        let weapons: &[Weapon] = debug::WEAPONS;
        let magics: &[Magic] = debug::MAGICS;
        let skills: &[Skill] = debug::SKILLS;
        let faction_builders: &[FactionBuilder] = debug::FACTION_BUILDERS;
        let unit_builders: &[UnitBuilder] = debug::UNIT_BUILDERS;
        let tile_builders: &[&[TileBuilder]] = debug::TILE_BUILDERS;

        Self { modifier_builders, effects, statuses, terrains, cities, weapons, magics, skills, faction_builders, unit_builders, tile_builders }
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

    pub fn get_tile_builders (&self) -> &[&[TileBuilder]] {
        self.tile_builders
    }
}
