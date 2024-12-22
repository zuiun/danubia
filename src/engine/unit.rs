use std::{cell::RefCell, cmp, rc::Rc, sync::atomic::Ordering};
use crate::engine::Lists;
use crate::engine::common::{Area, Capacity, Event, ID, IDS, Location, Modifier, Observer, Subject, Target, TYPE_UNIT, UnitStatisticTypes, Unique, WeaponStatisticTypes};
use crate::engine::event::{EVENT_MAP_GET_SUPPLY, EVENT_UNIT_DIED, EVENT_UNIT_SET_SUPPLY, VALUE_NOTIFICATION};
use crate::engine::map::Map;

type WeaponStatistics = [u8; WeaponStatisticTypes::Length as usize];
type UnitStatistics = [Capacity; UnitStatisticTypes::Length as usize];

const MRL_MAX: u16 = 100;
const HLT_MAX: u16 = 1000;
const SPL_MAX: u16 = 100;
const ATK_MAX: u16 = 200;
const DEF_MAX: u16 = 200;
const MAG_MAX: u16 = 200;
const MOV_MAX: u16 = 100;
const ORG_MAX: u16 = 200;
const DAMAGE_DIVIDEND_SPL: u16 = 3;
const DRAIN_SPL: u16 = 5;
const RECOVER_MRL: u16 = 1;
const RECOVER_POPULATION_DIVIDEND_HLT: u16 = 10;

trait Damage {
    fn get_dmg (&self) -> u16;
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
pub struct Faction {

}

#[derive (Debug)]
pub struct Unit {
    id: ID,
    lists: Rc<Lists>,
    map: Rc<Map>,
    statistics: UnitStatistics,
    modifiers: Vec<Modifier>,
    magic_ids: Vec<ID>,
    weapon_id: ID,
    faction_id: ID,
    supply_city_ids: Vec<ID>,
    observers: Vec<Rc<RefCell<dyn Observer>>>
}

impl UnitStatisticsBuilder {
    pub fn new (mrl: u16, hlt: u16, spl: u16, atk: u16, def: u16, mag: u16, mov: u16, org: u16) -> Self {
        assert! (mrl <= MRL_MAX);
        assert! (hlt <= HLT_MAX);
        assert! (spl <= SPL_MAX);
        assert! (org <= ORG_MAX);

        let mrl: Capacity = Capacity::Quantity (mrl, MRL_MAX);
        let hlt: Capacity = Capacity::Quantity (hlt, HLT_MAX);
        let spl: Capacity = Capacity::Quantity (spl, SPL_MAX);
        let atk: Capacity = Capacity::Constant (atk, ATK_MAX, atk);
        let def: Capacity = Capacity::Constant (def, DEF_MAX, def);
        let mag: Capacity = Capacity::Constant (mag, MAG_MAX, mag);
        let mov: Capacity = Capacity::Constant (mov, MOV_MAX, mov);
        let org: Capacity = Capacity::Quantity (org, ORG_MAX);

        Self { mrl, hlt, spl, atk, def, mag, mov, org }
    }

    pub fn build (self) -> UnitStatistics {
        [self.mrl, self.hlt, self.spl, self.atk, self.def, self.mag, self.mov, self.org]
    }
}

impl Weapon {
    pub const fn new (statistics: WeaponStatistics, dmg: u8, area: Area, range: u8) -> Self {
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
    pub fn new (lists: Rc<Lists>, statistics_builder: UnitStatisticsBuilder, magic_ids: Vec<ID>, weapon_id: ID, faction_id: ID, map: Rc<Map>) -> Self {
        let id: ID = Unit::assign_id ();
        let lists: Rc<Lists> = Rc::clone (&lists);
        let statistics: UnitStatistics = statistics_builder.build ();
        let modifiers: Vec<Modifier> = Vec::new ();
        let supply_city_ids: Vec<ID> = Vec::new ();
        let observers: Vec<Rc<RefCell<dyn Observer>>> = Vec::new ();

        Self { id, lists, map, statistics, modifiers, magic_ids, weapon_id, faction_id, supply_city_ids, observers }
    }

    fn get_statistic (&self, statistic: UnitStatisticTypes) -> (u16, u16) {
        assert! ((statistic as usize) < (UnitStatisticTypes::Length as usize));
        match statistic {
            UnitStatisticTypes::MRL => if let Capacity::Quantity (_, _) = self.statistics[UnitStatisticTypes::MRL as usize] {
                ()
            } else {
                panic! ("MRL should be a quantity");
            }
            UnitStatisticTypes::HLT => if let Capacity::Quantity (_, _) = self.statistics[UnitStatisticTypes::HLT as usize] {
                ()
            } else {
                panic! ("HLT should be a quantity");
            }
            UnitStatisticTypes::SPL => if let Capacity::Quantity (_, _) = self.statistics[UnitStatisticTypes::SPL as usize] {
                ()
            } else {
                panic! ("SPL should be a quantity");
            }
            UnitStatisticTypes::ATK => if let Capacity::Constant (_, _, _) = self.statistics[UnitStatisticTypes::ATK as usize] {
                ()
            } else {
                panic! ("ATK should be a constant");
            }
            UnitStatisticTypes::DEF => if let Capacity::Constant (_, _, _) = self.statistics[UnitStatisticTypes::DEF as usize] {
                ()
            } else {
                panic! ("DEF should be a constant");
            }
            UnitStatisticTypes::MAG => if let Capacity::Constant (_, _, _) = self.statistics[UnitStatisticTypes::MAG as usize] {
                ()
            } else {
                panic! ("MAG should be a constant");
            }
            UnitStatisticTypes::MOV => if let Capacity::Constant (_, _, _) = self.statistics[UnitStatisticTypes::MOV as usize] {
                ()
            } else {
                panic! ("MOV should be a constant");
            }
            UnitStatisticTypes::ORG => if let Capacity::Quantity (_, _) = self.statistics[UnitStatisticTypes::ORG as usize] {
                ()
            } else {
                panic! ("ORG should be a quantity");
            }
            _ => panic! ("Statistic not found")
        }

        match self.statistics[statistic as usize] {
            Capacity::Constant (c, _, b) => {
                (c, b)
            }
            Capacity::Quantity (c, m) => {
                (c, m)
            }
        }
    }

    fn set_statistic (&mut self, statistic: UnitStatisticTypes, value: u16) -> () {
        self.statistics[statistic as usize] = match self.statistics[statistic as usize] {
            Capacity::Constant (_, m, b) => {
                Capacity::Constant (value, m, b)
            }
            Capacity::Quantity (_, m) => {
                assert! (value < m);

                Capacity::Quantity (value, m)
            }
        };
    }

    fn change_statistic (&mut self, statistic: UnitStatisticTypes, change: u16, add: bool) -> () {
        let (current, maximum): (u16, u16) = match self.statistics[statistic as usize] {
            Capacity::Constant (c, m, _) => {
                (c, m)
            }
            Capacity::Quantity (c, m) => {
                (c, m)
            }
        };
        let value = if add {
            cmp::min (current + change, maximum)
        } else {
            current.checked_sub (change).unwrap_or (0)
        };

        self.set_statistic (statistic, value);
    }

    pub fn start_turn (&mut self) -> () {
        self.modifiers.retain (|m| m.get_duration () > 0);
        // TODO: apply all constant passive skills

        let location: &Location = self.map.get_unit_location (&self.id)
                .expect (&format! ("Location not found for unit {}", self.id));

        if self.map.is_impassable (location) {
            self.die ();
        }
    }

    fn calculate_damage_weapon (&self, other: &Unit, magic_id: Option<ID>) -> u16 {
        match magic_id {
            Some (m) => {
                let magic: &Magic = self.lists.get_magic (&m);
                let mag_self: u16 = self.get_statistic (UnitStatisticTypes::MAG).0;
                let mag_other: u16 = other.get_statistic (UnitStatisticTypes::MAG).0;
                let damage: f32 = (mag_self as f32) / (mag_other as f32);
                let multiplier: f32 = magic.get_dmg () as f32;

                (damage * multiplier) as u16
            }
            None => {
                let weapon: &Weapon = self.lists.get_weapon (&self.weapon_id);
                let atk_self: u16 = self.get_statistic (UnitStatisticTypes::ATK).0;
                let def_other: u16 = other.get_statistic (UnitStatisticTypes::DEF).0;
                let spl_self: (u16, u16) = self.get_statistic (UnitStatisticTypes::SPL);
                let spl_other: (u16, u16) = other.get_statistic (UnitStatisticTypes::SPL);
                let atk_final: u16 = atk_self + weapon.get_dmg ();
                let multiplier_other: f32 = (spl_other.0 as f32) / (spl_other.1 as f32);
                let def_final: u16 = ((def_other as f32) * multiplier_other) as u16;
                let damage: u16 = cmp::max (atk_final.checked_sub (def_final).unwrap_or (1), 1);
                let multiplier_self: f32 = (spl_self.0 as f32) / (spl_self.1 as f32);

                ((damage as f32) * multiplier_self) as u16
            }
        }
    }

    fn calculate_damage_bonus (&self, other: &Unit) -> u16 {
        let weapon: &Weapon = self.lists.get_weapon (&self.weapon_id);
        let mag_self: u16 = self.get_statistic (UnitStatisticTypes::MAG).0;
        let mag_other: u16 = other.get_statistic (UnitStatisticTypes::MAG).0;
        let damage: u16 = cmp::max (mag_self.checked_sub (mag_other).unwrap_or (1), 1);
        let multiplier: u16 = ((weapon.get_statistic (WeaponStatisticTypes::DCY) + 1) as u16) * 2;

        damage * multiplier
    }

    fn calculate_damage_multiplier (&self) -> f32 {
        let hlt: (u16, u16) = self.get_statistic (UnitStatisticTypes::HLT);
        let org: (u16, u16) = self.get_statistic (UnitStatisticTypes::ORG);
        let multiplier_hlt: f32 = (hlt.0 as f32) / (hlt.1 as f32);
        let multiplier_org: f32 = 1.0 + ((org.0 as f32) / (org.1 as f32));

        multiplier_hlt * multiplier_org
    }

    fn die (&mut self) -> () {
        self.notify ((EVENT_UNIT_DIED, VALUE_NOTIFICATION));
        // TODO: ???
    }

    pub fn attack_character (&mut self, other: &mut Unit, magic_id: Option<ID>) -> bool {
        let weapon: &Weapon = self.lists.get_weapon (&self.weapon_id);
        let damage_weapon: u16 = self.calculate_damage_weapon (other, magic_id);
        let damage_bonus: u16 = self.calculate_damage_bonus (other);
        let multiplier: f32 = self.calculate_damage_multiplier ();
        let damage_base: u16 = (((damage_weapon + damage_bonus) as f32) * multiplier) as u16;
        let damage_mrl: u16 = damage_base + (weapon.get_statistic (WeaponStatisticTypes::SLH) as u16);
        let damage_hlt: u16 = damage_base * ((weapon.get_statistic (WeaponStatisticTypes::PRC) + 1) as u16);

println!("{:?}",weapon);
if magic_id.is_some () { println!("{:?}",magic_id.unwrap()) }
println!("{},{}",damage_mrl,damage_hlt);

        other.take_damage (damage_mrl, damage_hlt)
    }

    fn take_damage (&mut self, damage_mrl: u16, damage_hlt: u16) -> bool {
        let damage_spl: u16 = damage_mrl / DAMAGE_DIVIDEND_SPL;

        self.change_statistic (UnitStatisticTypes::MRL, damage_mrl, false);
        self.change_statistic (UnitStatisticTypes::HLT, damage_hlt, false);
        self.change_statistic (UnitStatisticTypes::SPL, damage_spl, false);

println!("{},{}",
self.get_statistic (UnitStatisticTypes::MRL).0,
self.get_statistic (UnitStatisticTypes::HLT).0);

        if self.get_statistic (UnitStatisticTypes::HLT).0 == 0 {
            self.die ();
            
            true
        } else {
            false
        }
    }

    pub fn end_turn (&mut self) -> () {
        for modifier in self.modifiers.iter_mut () {
            modifier.dec_duration ();
        }

        self.supply_city_ids.clear ();
        self.notify ((EVENT_MAP_GET_SUPPLY, VALUE_NOTIFICATION));

println!("{:?}",self.supply_city_ids);
        if self.supply_city_ids.len () > 0 {
            let mut recover_hlt: u16 = 0;
            let mut recover_spl: u16 = 0;

            for supply_city_id in &self.supply_city_ids {
                let population_city: u16 = self.lists.get_city (supply_city_id).get_population ();
                let factories_city: u16 = self.lists.get_city (supply_city_id).get_factories ();
                let farms_city: u16 = self.lists.get_city (supply_city_id).get_farms ();
                let modifier_hlt: f32 = (farms_city as f32) / (factories_city as f32);
                let change_hlt: u16 = (((population_city / RECOVER_POPULATION_DIVIDEND_HLT) as f32) * modifier_hlt) as u16;
                let change_spl: u16 = factories_city;

                recover_hlt += change_hlt;
                recover_spl += change_spl;
            }

            self.change_statistic (UnitStatisticTypes::MRL, RECOVER_MRL, true);
            self.change_statistic (UnitStatisticTypes::HLT, recover_hlt, true);
            self.change_statistic (UnitStatisticTypes::SPL, recover_spl, false);
        } else {
            self.change_statistic (UnitStatisticTypes::SPL, DRAIN_SPL, false);
        }
    }

    pub fn add_modifier (&mut self, modifier: Modifier) -> bool {
        self.modifiers.push (modifier);

        // TODO: Apply modifier effect
        todo!()
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }
}

impl Damage for Weapon {
    fn get_dmg (&self) -> u16 {
        self.dmg as u16
    }
}

impl Damage for Magic {
    fn get_dmg (&self) -> u16 {
        self.dmg as u16
    }
}

impl Unique for Unit {
    fn assign_id () -> ID {
        IDS[TYPE_UNIT as usize].fetch_add (1, Ordering::SeqCst)
    }

    fn get_id (&self) -> ID {
        self.id
    }

    fn get_type (&self) -> ID {
        TYPE_UNIT
    }
}

impl Observer for Unit {
    fn update (&mut self, event: Event) -> () {
        match event {
            (EVENT_UNIT_SET_SUPPLY, v) => {
                let city_id: ID = v as ID;
                let unit_id: ID = (v >> 8) as ID;

                if unit_id == self.id {
                    self.supply_city_ids.push (city_id);
                }
            }
            _ => ()
        }
    }
}

impl Subject for Unit {
    fn add_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        let observer: Rc<RefCell<dyn Observer>> = Rc::clone (&observer);

        self.observers.push (observer);
    }

    fn remove_observer (&mut self, observer: Rc<RefCell<dyn Observer>>) -> () {
        unimplemented! ()
    }

    fn notify (&self, event: Event) -> () {
        for observer in self.observers.iter () {
            observer.borrow_mut ().update (event);
        }
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::engine::tests::generate_lists;
    use crate::engine::map::tests::generate_map;

    fn generate_units () -> (Unit, Unit, Unit) {
        let lists: Rc<Lists> = generate_lists ();
        let map: Map = generate_map ();
        let map: Rc<Map> = Rc::new (map);

        let statistics_character_1: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_1: Unit = Unit::new (Rc::clone (&lists), statistics_character_1, vec![], 0, 0, Rc::clone (&map));

        let statistics_character_2: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_2: Unit = Unit::new (Rc::clone (&lists), statistics_character_2, vec![], 1, 0, Rc::clone (&map));

        let statistics_character_3: UnitStatisticsBuilder = UnitStatisticsBuilder::new (100, 1000, 100, 20, 20, 20, 10, 100);
        let character_3: Unit = Unit::new (Rc::clone (&lists), statistics_character_3, vec![], 2, 0, Rc::clone (&map));

        (character_1, character_2, character_3)
    }

    #[test]
    fn character_attack_character () {
        let (mut character_1, mut character_2, mut character_3) = generate_units ();

        character_1.attack_character (&mut character_2, None);
        character_2.attack_character (&mut character_3, None);
        character_3.attack_character (&mut character_1, None);
        assert! (false);
    }
}
