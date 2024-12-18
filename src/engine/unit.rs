use std::{cell::RefCell, cmp, collections::HashMap, rc::Rc};
use crate::engine::common::{Area, Capacity, Event, ID, Modifier, Observer, Subject, Target, UnitStatisticTypes, Value, WeaponStatisticTypes};
use crate::engine::event::{SET_ENCIRCLED_EVENT, UNIT_DIED_EVENT, UNIT_TYPE};
use crate::engine::map::Map;

const MRL_MAX: Value = 100;
const HLT_MAX: Value = 1000;
const SPL_MAX: Value = 100;
const ORG_MAX: Value = 200;

type WeaponStatistics = [u8; WeaponStatisticTypes::Length as usize];
type UnitStatistics = [Capacity; UnitStatisticTypes::Length as usize];

trait Damage {
    fn get_dmg (&self) -> u8;
}

#[derive (Debug)]
pub struct UnitStatisticsBuilder {
    mrl: Capacity,
    hlt: Capacity,
    spl: Capacity,
    atk: Capacity,
    def: Capacity,
    mag: Capacity,
    mov: Capacity,
    org: Capacity
}

#[derive (Debug)]
pub struct Weapon {
    statistics: WeaponStatistics,
    dmg: u8,
    area: Area,
    range: u8
}

#[derive (Debug)]
pub struct Magic {
    // TODO: effects
    dmg: u8,
    area: Area,
    range: u8
}

#[derive (Debug)]
pub struct Skill {
    area: Area,
    range: u8,
    target: Target,
    is_passive: bool,
    is_constant: bool
}

#[derive (Debug)]
pub struct Unit {
    map: Rc<Map>,
    weapons: Rc<HashMap<ID, Weapon>>,
    magics: Rc<HashMap<ID, Magic>>,
    statistics: UnitStatistics,
    modifiers: Vec<Modifier>,
    magic_ids: Vec<ID>,
    weapon_id: ID,
    faction_id: ID,
    observers: Vec<Rc<RefCell<dyn Observer>>>,
    is_encircled: bool
}

pub struct UnitBuilder {
    statistics_builder: UnitStatisticsBuilder,
    magic_ids: Vec<ID>,
    weapon_id: ID,
    faction_id: ID
}

impl UnitStatisticsBuilder {
    pub fn new (mrl: Value, hlt: Value, spl: Value, atk: Value, def: Value, mag: Value, mov: Value, org: Value) -> Self {
        assert! (mrl <= MRL_MAX);
        assert! (hlt <= HLT_MAX);
        assert! (spl <= SPL_MAX);
        assert! (org <= ORG_MAX);

        let mrl: Capacity = Capacity::Quantity (mrl, MRL_MAX);
        let hlt: Capacity = Capacity::Quantity (hlt, HLT_MAX);
        let spl: Capacity = Capacity::Quantity (spl, SPL_MAX);
        let atk: Capacity = Capacity::Constant (atk, atk);
        let def: Capacity = Capacity::Constant (def, def);
        let mag: Capacity = Capacity::Constant (mag, mag);
        let mov: Capacity = Capacity::Constant (mov, mov);
        let org: Capacity = Capacity::Quantity (org, ORG_MAX);

        Self { mrl, hlt, spl, atk, def, mag, mov, org }
    }

    pub fn build (self) -> UnitStatistics {
        [self.mrl, self.hlt, self.spl, self.atk, self.def, self.mag, self.mov, self.org]
    }
}

impl Weapon {
    pub fn new (statistics: WeaponStatistics, dmg: u8, area: Area, range: u8) -> Self {
        Self { statistics, dmg, area, range }
    }

    pub fn get_statistic (&self, statistic: WeaponStatisticTypes) -> u8 {
        self.statistics[statistic as usize]
    }
}

impl Magic {
    pub fn new (dmg: u8, area: Area, range: u8) -> Self {
        Self { dmg, area, range }
    }
}

impl Skill {

}

impl Unit {
    pub fn get_weapon (&self) -> &Weapon {
        self.weapons.get (&self.weapon_id)
                .expect (&format! ("Weapon {} not found", self.weapon_id))
    }

    pub fn get_magic (&self, magic_id: ID) -> &Magic {
        self.magics.get (&magic_id)
                .expect (&format! ("Magic {} not found", magic_id))
    }

    fn set_statistic (&mut self, statistic: UnitStatisticTypes, value: u16) -> () {
        self.statistics[statistic as usize] = match self.statistics[statistic as usize] {
            Capacity::Constant (_, b) => {
                Capacity::Quantity (value, b)
            }
            Capacity::Quantity (_, m) => {
                assert! (value < m);

                Capacity::Quantity (value, m)
            }
        };
    }

    pub fn start_turn (&mut self) -> () {
        self.modifiers.retain (|m| m.get_duration () > 0);
        // apply all constant passive skills
        // die if on impassable terrain
    }

    fn calculate_damage_weapon (&self, other: &Unit, magic_id: Option<ID>) -> Value {
        match magic_id {
            Some (m) => {
                let magic: &Magic = self.get_magic (m);
                let mag: Value = if let Capacity::Constant (m, _) = self.statistics[UnitStatisticTypes::MAG as usize] {
                    m
                } else {
                    panic! ("MAG should be a constant");
                };
                let damage: f32 = if let Capacity::Constant (m, _) = other.statistics[UnitStatisticTypes::MAG as usize] {
                    (mag as f32) / (m as f32)
                } else {
                    panic! ("MAG should be a constant");
                };
                let multiplier: f32 = magic.get_dmg () as f32;

                (damage * multiplier) as Value
            }
            None => {
                let weapon: &Weapon = self.get_weapon ();
                let atk: Value = if let Capacity::Constant (a, _) = self.statistics[UnitStatisticTypes::ATK as usize] {
                    (weapon.get_dmg () as Value) + a
                } else {
                    panic! ("ATK should be a constant");
                };
                let damage: Value = if let Capacity::Constant (d, _) = other.statistics[UnitStatisticTypes::DEF as usize] {
                    cmp::max (atk.checked_sub (d).unwrap_or (1), 1)
                } else {
                    panic! ("DEF should be a constant");
                };
                let multiplier: f32 = if let Capacity::Quantity (s, m) = self.statistics[UnitStatisticTypes::SPL as usize] {
                    (s as f32) / (m as f32)
                } else {
                    panic! ("SPL should be a quantity");
                };

                ((damage as f32) * multiplier) as Value
            }
        }
    }

    fn calculate_damage_bonus (&self, other: &Unit) -> Value {
        let weapon: &Weapon = self.get_weapon ();
        let mag: Value = if let Capacity::Constant (m, _) = self.statistics[UnitStatisticTypes::MAG as usize] {
            m
        } else {
            panic! ("MAG should be a constant");
        };
        let damage: Value = if let Capacity::Constant (m, _) = other.statistics[UnitStatisticTypes::MAG as usize] {
            cmp::max (mag.checked_sub (m).unwrap_or (1), 1)
        } else {
            panic! ("MAG should be a constant");
        };
        let multiplier: Value = ((weapon.get_statistic (WeaponStatisticTypes::DCY) + 1) as Value) * 2;

        damage * multiplier
    }

    fn calculate_damage_multiplier (&self) -> f32 {
        let multiplier_hlt: f32 = if let Capacity::Quantity (h, m) = self.statistics[UnitStatisticTypes::HLT as usize] {
            (h as f32) / (m as f32)
        } else {
            panic! ("HLT should be a quantity");
        };
        let multiplier_org: f32 = if let Capacity::Quantity (o, m) = self.statistics[UnitStatisticTypes::ORG as usize] {
            1.0 + ((o as f32) / (m as f32))
        } else {
            panic! ("ORG should be a quantity");
        };

        multiplier_hlt * multiplier_org
    }

    fn die (&mut self) -> () {
        // ???
    }

    pub fn attack_character (&mut self, other: &mut Unit, magic_id: Option<ID>) -> bool {
        let weapon: &Weapon = self.get_weapon ();
        let damage_weapon: Value = self.calculate_damage_weapon (other, magic_id);
        let damage_bonus: Value = self.calculate_damage_bonus (other);
        let multiplier: f32 = self.calculate_damage_multiplier ();
        let damage_base: Value = (((damage_weapon + damage_bonus) as f32) * multiplier) as Value;
        let damage_mrl: Value = damage_base + (weapon.get_statistic (WeaponStatisticTypes::SLH) as Value);
        let damage_hlt: Value = damage_base * ((weapon.get_statistic (WeaponStatisticTypes::PRC) + 1) as Value);

println!("{:?}",weapon);
if magic_id.is_some () { println!("{:?}",magic_id.unwrap()) }
println!("{},{}",damage_mrl,damage_hlt);

        other.take_damage (damage_mrl, damage_hlt)
    }

    fn take_damage (&mut self, damage_mrl: Value, damage_hlt: Value) -> bool {
        let mrl: Value = if let Capacity::Quantity (m, _) = self.statistics[UnitStatisticTypes::MRL as usize] {
            m
        } else {
            panic! ("MRL should be a quantity");
        };
        let mrl: Value = mrl.checked_sub (damage_mrl).unwrap_or (0);
        let hlt: Value = if let Capacity::Quantity (h, _) = self.statistics[UnitStatisticTypes::HLT as usize] {
            h
        } else {
            panic! ("HLT should be a quantity");
        };
        let hlt: Value = hlt.checked_sub (damage_hlt).unwrap_or (0);
        let spl: Value = if let Capacity::Quantity (s, _) = self.statistics[UnitStatisticTypes::SPL as usize] {
            s
        } else {
            panic! ("SPL should be a quantity");
        };
        let spl: Value = spl.checked_sub (damage_mrl / 3).unwrap_or (0);

        self.set_statistic (UnitStatisticTypes::MRL, mrl);
        self.set_statistic (UnitStatisticTypes::HLT, hlt);
        self.set_statistic (UnitStatisticTypes::SPL, spl);

        if hlt == 0 {
            self.die ();
        }

println!("{},{}",mrl,hlt);
        hlt == 0
    }

    pub fn end_turn (&mut self) -> () {
        let _ = self.modifiers.iter_mut ().map (|m| m.dec_duration ()).collect::<Vec<_>> ();
        
        if self.is_encircled {
            // TODO: Drain supplies
        } else {
            // TODO: Recover health and morale
        }
    }

    pub fn add_modifier (&mut self, modifier: &Modifier) -> bool {
        todo!()
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }
}

impl UnitBuilder {
    pub fn new (statistics_builder: UnitStatisticsBuilder, magic_ids: Vec<ID>, weapon_id: ID, faction_id: ID) -> Self {
        Self { statistics_builder, magic_ids, weapon_id, faction_id }
    }

    pub fn build (self, map: Rc<Map>, weapons: Rc<HashMap<ID, Weapon>>, magics: Rc<HashMap<ID, Magic>>) -> Unit {
        let statistics: UnitStatistics = self.statistics_builder.build ();
        let modifiers: Vec<Modifier> = Vec::new ();
        let observers: Vec<Rc<RefCell<dyn Observer>>> = Vec::new ();
        let is_encircled: bool = false;

        Unit { map, weapons, magics, statistics, modifiers, magic_ids: self.magic_ids, weapon_id: self.weapon_id, faction_id: self.faction_id, observers, is_encircled }
    }
}

impl Damage for Weapon {
    fn get_dmg (&self) -> u8 {
        self.dmg
    }
}

impl Damage for Magic {
    fn get_dmg (&self) -> u8 {
        self.dmg
    }
}

impl Observer for Unit {
    fn update (&mut self, event: Event) -> () {
        match event {
            (SET_ENCIRCLED_EVENT, v) => self.is_encircled = v > 0,
            _ => ()
        }
    }

    fn get_type (&self) -> ID {
        UNIT_TYPE
    }
}

impl Subject for Unit {
    fn add_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        let observer: Rc<RefCell<dyn Observer>> = Rc::clone (&observer);

        self.observers.push (observer);
    }

    fn remove_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        todo!()
    }

    fn notify (&self, event: Event) -> () {
        self.observers.iter ().map (|o| o.borrow_mut ().update (event)).collect () // Ignore borrow checking, since single-threaded
    }
}

#[cfg (test)]
mod tests {
    use std::collections::HashMap;
    use crate::engine::map::{Terrain, TileBuilder};
    use super::*;

    fn generate_terrains () -> HashMap<ID, Terrain> {
        let grass: Terrain = Terrain::new (Vec::new (), 1);
        let dirt: Terrain = Terrain::new (Vec::new (), 2);
        let stone: Terrain = Terrain::new (Vec::new (), 0);
        let mut terrains: HashMap<ID, Terrain> = HashMap::new ();

        terrains.insert (0, grass);
        terrains.insert (1, dirt);
        terrains.insert (2, stone);

        terrains
    }

    fn generate_tile_builders () -> Vec<Vec<TileBuilder>> {
        vec![
            vec![TileBuilder::new (0, 0, None), TileBuilder::new (0, 1, None), TileBuilder::new (0, 0, None)],
            vec![TileBuilder::new (1, 2, None), TileBuilder::new (1, 1, None), TileBuilder::new (2, 0, None)]
        ]
    }

    fn generate_unit_factions () -> HashMap<ID, ID> {
        let mut unit_factions: HashMap<ID, ID> = HashMap::new ();

        unit_factions.insert (0, 1);
        unit_factions.insert (1, 1);
        unit_factions.insert (2, 2);
        unit_factions.insert (3, 3);

        unit_factions
    }

    fn generate_map () -> Map {
        let terrains: HashMap<ID, Terrain> = generate_terrains ();
        let tile_map_builder: Vec<Vec<TileBuilder>> = generate_tile_builders ();
        let unit_factions: HashMap<ID, ID> = generate_unit_factions ();

        Map::new (terrains, tile_map_builder, unit_factions)
    }

    fn generate_weapons () -> HashMap<ID, Weapon> {
        let mut weapons: HashMap<ID, Weapon> = HashMap::new ();

        let sword_statistics: WeaponStatistics = [1, 1, 0];
        let sword = Weapon::new (sword_statistics, 2, Area::Single, 1);

        let spear_statistics: WeaponStatistics = [0, 2, 0];
        let spear = Weapon::new (spear_statistics, 2, Area::Path (1), 2);

        let book_statistics: WeaponStatistics = [1, 0, 1];
        let book = Weapon::new (book_statistics, 1, Area::Radial (2), 2);

        weapons.insert (0, sword);
        weapons.insert (1, spear);
        weapons.insert (2, book);

        weapons
    }

    fn generate_magics () -> HashMap<ID, Magic> {
        let mut magics: HashMap<ID, Magic> = HashMap::new ();

        magics
    }

    fn generate_characters () -> (Unit, Unit, Unit) {
        let map: Map = generate_map ();
        let map: Rc<Map> = Rc::new (map);
        let weapons: HashMap<ID, Weapon> = generate_weapons ();
        let weapons: Rc<HashMap<ID, Weapon>> = Rc::new (weapons);
        let magics: HashMap<ID, Magic> = generate_magics ();
        let magics: Rc<HashMap<ID, Magic>> = Rc::new (magics);

        let statistics_character_1: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_1: UnitBuilder = UnitBuilder::new (statistics_character_1, vec![], 0, 0);
        let character_1: Unit = character_1.build (Rc::clone (&map), Rc::clone (&weapons), Rc::clone (&magics));

        let statistics_character_2: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_2: UnitBuilder = UnitBuilder::new (statistics_character_2, vec![], 0, 1);
        let character_2: Unit = character_2.build (Rc::clone (&map), Rc::clone (&weapons), Rc::clone (&magics));

        let statistics_character_3: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_3: UnitBuilder = UnitBuilder::new (statistics_character_3, vec![], 0, 2);
        let character_3: Unit = character_3.build (Rc::clone (&map), Rc::clone (&weapons), Rc::clone (&magics));

        (character_1, character_2, character_3)
    }

    #[test]
    fn character_attack_character () {
        let (mut character_1, mut character_2, mut character_3) = generate_characters ();

        character_1.attack_character (&mut character_2, None);
        character_2.attack_character (&mut character_3, None);
        character_3.attack_character (&mut character_1, None);
        assert! (false);
    }
}
