use crate::character::{FactionBuilder, Magic, Skill, UnitBuilder, Weapon};
use crate::common::ID;
use crate::dynamic::{Attribute, Effect, Modifier};
use crate::map::{City, Location, Terrain, TileBuilder};
use super::debug;

#[derive (Debug)]
pub struct Scene {
    // Objects
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
    unit_locations: &'static [Option<Location>],
    // Textures
    textures_terrain: &'static [&'static str],
    textures_unit: &'static [&'static str],
}

impl Scene {
    #[allow (clippy::too_many_arguments)]
    pub fn new (modifiers: &'static [Modifier], effects: &'static [Effect], attributes: &'static [Attribute], terrains: &'static [Terrain], cities: &'static [City], weapons: &'static [Weapon], magics: &'static [Magic], skills: &'static [Skill], faction_builders: &'static [FactionBuilder], unit_builders: &'static [UnitBuilder], tile_builders: &'static [&'static [TileBuilder]], unit_locations: &'static [Option<Location>], textures_terrain: &'static [&'static str], textures_unit: &'static [&'static str]) -> Self {
        Self { modifiers, effects, attributes, terrains, cities, weapons, magics, skills, faction_builders, unit_builders, tile_builders, unit_locations, textures_terrain, textures_unit }
    }

    #[allow (clippy::too_many_arguments)]
    pub fn debug () -> Self {
        let modifiers: &[Modifier] = debug::objects::MODIFIERS;
        let effects: &[Effect] = debug::objects::EFFECTS;
        let attributes: &[Attribute] = debug::objects::ATTRIBUTES;
        let terrains: &[Terrain] = debug::objects::TERRAINS;
        let cities: &[City] = debug::objects::CITIES;
        let weapons: &[Weapon] = debug::objects::WEAPONS;
        let magics: &[Magic] = debug::objects::MAGICS;
        let skills: &[Skill] = debug::objects::SKILLS;
        let faction_builders: &[FactionBuilder] = debug::objects::FACTION_BUILDERS;
        let unit_builders: &[UnitBuilder] = debug::objects::UNIT_BUILDERS;
        let tile_builders: &[&[TileBuilder]] = debug::objects::TILE_BUILDERS;
        let unit_locations: &[Option<Location>] = debug::objects::UNIT_LOCATIONS;

        let textures_terrain: &[&str] = debug::textures::TERRAINS;
        let textures_unit: &[&str] = debug::textures::UNITS;

        Self { modifiers, effects, attributes, terrains, cities, weapons, magics, skills, faction_builders, unit_builders, tile_builders, unit_locations, textures_terrain, textures_unit }
    }

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

    pub fn get_unit_location (&self, unit_id: &ID) -> &Option<Location> {
        assert! (*unit_id < self.unit_locations.len ());

        &self.unit_locations[*unit_id]
    }

    pub fn unit_locations_iter (&self) -> impl Iterator<Item = &Option<Location>> {
        self.unit_locations.iter ()
    }

    pub fn textures_terrain_iter (&self) -> impl Iterator<Item = &&str> {
        self.textures_terrain.iter ()
    }

    pub fn textures_unit_iter (&self) -> impl Iterator<Item = &&str> {
        self.textures_unit.iter ()
    }
}

impl Default for Scene {
    fn default () -> Self {
        let modifiers: &[Modifier] = debug::objects::MODIFIERS;
        let effects: &[Effect] = debug::objects::EFFECTS;
        let attributes: &[Attribute] = debug::objects::ATTRIBUTES;
        let terrains: &[Terrain] = debug::objects::TERRAINS;
        let cities: &[City] = debug::objects::CITIES;
        let weapons: &[Weapon] = debug::objects::WEAPONS;
        let magics: &[Magic] = debug::objects::MAGICS;
        let skills: &[Skill] = debug::objects::SKILLS;
        let faction_builders: &[FactionBuilder] = debug::objects::FACTION_BUILDERS;
        let unit_builders: &[UnitBuilder] = debug::objects::UNIT_BUILDERS;
        let tile_builders: &[&[TileBuilder]] = debug::objects::TILE_BUILDERS;
        let unit_locations: &[Option<Location>] = debug::objects::UNIT_LOCATIONS;

        let textures_terrain: &[&str] = debug::textures::TERRAINS;
        let textures_unit: &[&str] = debug::textures::UNITS;

        Self { modifiers, effects, attributes, terrains, cities, weapons, magics, skills, faction_builders, unit_builders, tile_builders, unit_locations, textures_terrain, textures_unit }
    }
}
