use super::{Element, Magic, Skill, Weapon, WeaponStatistic};
use self::UnitStatistic::{MRL, HLT, SPL, ATK, DEF, MAG, MOV, ORG};
use crate::common::{Capacity, FACTOR_MAGIC, FACTOR_SKILL, FACTOR_WAIT, ID, Scene, Target, Timed};
use crate::dynamic::{Appliable, AppliableKind, Applier, Attribute, Dynamic, Effect, Modifier, StatisticKind, Trigger};
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

const PERCENT_1: u16 = 1_0;
#[allow (clippy::inconsistent_digit_grouping)]
const PERCENT_100: u16 = 100_0;
const MRL_MAX: u16 = PERCENT_100; // 100.0%
const HLT_MAX: u16 = 1000; // 1000
const SPL_MAX: u16 = PERCENT_100; // 100.0%
const ATK_MAX: u16 = 200; // 200
const DEF_MAX: u16 = 200; // 200
const MAG_MAX: u16 = 200; // 200
const MOV_MAX: u16 = 100; // 100
const ORG_MAX: u16 = 2 * PERCENT_100; // 200.0%
const DRAIN_SPL: f32 = 5_0.0; // 5.0%
#[allow (clippy::inconsistent_digit_grouping)]
const RECOVER_MRL: u16 = 5_0; // 5.0%
const DRAIN_HLT: u16 = 4; // 4
#[allow (clippy::inconsistent_digit_grouping)]
const THRESHOLD_RETREAT_MRL: u16 = 40_0; // 40.0%
#[allow (clippy::inconsistent_digit_grouping)]
const THRESHOLD_ROUT_MRL: u16 = 20_0; // 20.0%
const FACTOR_FIGHT: u16 = 1;
const FACTOR_RETREAT: u16 = 2;
const FACTOR_ROUT: u16 = 4;
const THRESHOLD_SKILL_PASSIVE: usize = 1; // TODO: needs to be balanced
const UNIT_STATISTICS: [UnitStatistic; UnitStatistic::Length as usize] = [
    MRL,
    HLT,
    SPL,
    ATK,
    DEF,
    MAG,
    MOV,
    ORG,
];

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum UnitStatistic {
    MRL, // morale - willingness to fight (permillage)
    HLT, // manpower - number of soldiers
    SPL, // supply - proportion of soldiers equipped (permillage)
    ATK, // attack – physical damage
    DEF, // defence – physical resistance
    MAG, // magic – magical damage and resistance
    MOV, // manoeuvre – speed and movement
    ORG, // cohesion – modifier for physical damage, resistance, and leader passive (permillage)
    Length,
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub struct UnitStatistics ([Capacity; UnitStatistic::Length as usize]);

impl UnitStatistics {
    #[allow (clippy::too_many_arguments)]
    pub const fn new (mrl: u16, hlt: u16, spl: u16, atk: u16, def: u16, mag: u16, mov: u16, org: u16) -> Self {
        assert! (mrl <= MRL_MAX);
        assert! (hlt <= HLT_MAX);
        assert! (spl <= SPL_MAX);
        assert! (atk <= ATK_MAX);
        assert! (def <= DEF_MAX);
        assert! (mag <= MAG_MAX);
        assert! (mov <= MOV_MAX);
        assert! (org <= ORG_MAX);

        let mrl: Capacity = Capacity::Quantity (mrl, MRL_MAX);
        let hlt: Capacity = Capacity::Quantity (hlt, HLT_MAX);
        let spl: Capacity = Capacity::Quantity (spl, SPL_MAX);
        let atk: Capacity = Capacity::Constant (atk, ATK_MAX, atk);
        let def: Capacity = Capacity::Constant (def, DEF_MAX, def);
        let mag: Capacity = Capacity::Constant (mag, MAG_MAX, mag);
        let mov: Capacity = Capacity::Constant (mov, MOV_MAX, mov);
        let org: Capacity = Capacity::Constant (org, ORG_MAX, org);
        let statistics: [Capacity; UnitStatistic::Length as usize] = [mrl, hlt, spl, atk, def, mag, mov, org];

        Self (statistics)
    }

    fn validate_statistic (&self, statistic: UnitStatistic) -> bool {
        match statistic {
            MRL => matches! (self.0[MRL as usize], Capacity::Quantity ( .. )),
            HLT => matches! (self.0[HLT as usize], Capacity::Quantity ( .. )),
            SPL => matches! (self.0[SPL as usize], Capacity::Quantity ( .. )),
            ATK => matches! (self.0[ATK as usize], Capacity::Constant ( .. )),
            DEF => matches! (self.0[DEF as usize], Capacity::Constant ( .. )),
            MAG => matches! (self.0[MAG as usize], Capacity::Constant ( .. )),
            MOV => matches! (self.0[MOV as usize], Capacity::Constant ( .. )),
            ORG => matches! (self.0[ORG as usize], Capacity::Constant ( .. )),
            _ => panic! ("Statistic not found"),
        }
    }

    fn get_statistic (&self, statistic: UnitStatistic) -> (u16, u16) {
        assert! (self.validate_statistic (statistic));

        match self.0[statistic as usize] {
            Capacity::Constant (c, _, b) => (c, b),
            Capacity::Quantity (c, m) => (c, m),
        }
    }

    fn set_statistic (&mut self, statistic: UnitStatistic, value: u16) {
        assert! (self.validate_statistic (statistic));

        self.0[statistic as usize] = match self.0[statistic as usize] {
            Capacity::Constant (_, m, b) => {
                assert! (value <= m);

                Capacity::Constant (value, m, b)
            }
            Capacity::Quantity (_, m) => {
                assert! (value <= m);

                Capacity::Quantity (value, m)
            }
        };
    }

    fn change_statistic_flat (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) {
        assert! (self.validate_statistic (statistic));

        let (current, maximum): (u16, u16) = match self.0[statistic as usize] {
            Capacity::Constant (c, m, _) => (c, m),
            Capacity::Quantity (c, m) => (c, m),
        };
        let value: u16 = if is_add {
            u16::min (current + change, maximum)
        } else {
            current.saturating_sub (change)
        };

        self.set_statistic (statistic, value);
    }

    fn change_statistic_percentage (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) {
        assert! (self.validate_statistic (statistic));

        let base: f32 = match self.0[statistic as usize] {
            Capacity::Constant (_, _, b) => b,
            Capacity::Quantity (_, m) => m,
        } as f32;
        let change: f32 = (change as f32) / 100.0;
        let change: u16 = (base * change) as u16;

        self.change_statistic_flat (statistic, change, is_add);
    }

    pub fn calculate_damage (attacker: &Self, defender: &Self, weapon: &Weapon) -> (u16, u16, u16) {
        let dmg_weapon: u16 = weapon.get_statistic (WeaponStatistic::DMG);
        let slh_weapon: u16 = weapon.get_statistic (WeaponStatistic::SLH);
        let prc_weapon: u16 = weapon.get_statistic (WeaponStatistic::PRC);
        let dcy_weapon: u16 = weapon.get_statistic (WeaponStatistic::DCY);

        let hlt_attacker: (u16, u16) = attacker.get_statistic (HLT);
        let spl_attacker: (u16, u16) = attacker.get_statistic (SPL);
        let atk_attacker: u16 = attacker.get_statistic (ATK).0;
        let mag_attacker: u16 = attacker.get_statistic (MAG).0;
        let org_attacker: u16 = attacker.get_statistic (ORG).0;

        let mrl_defender: (u16, u16) = defender.get_statistic (MRL);
        let spl_defender: (u16, u16) = defender.get_statistic (SPL);
        let def_defender: u16 = defender.get_statistic (DEF).0;
        let mag_defender: u16 = defender.get_statistic (MAG).0;
        let org_defender: u16 = defender.get_statistic (ORG).0;

        let damage_weapon: f32 = {
            let damage: f32 = (atk_attacker + dmg_weapon) as f32;
            let factor: f32 = (spl_attacker.0 as f32) / (spl_attacker.1 as f32);

            damage * factor
        };
        let damage_magic: u16 = {
            let add: u16 = (dcy_weapon * 2) + 1;
            let damage: u16 = u16::max ((mag_attacker + add).saturating_sub (mag_defender), 1);

            damage * add
        };
        let factor: f32 = {
            let factor_mrl: f32 = 1.0 - (mrl_defender.0 as f32) / (mrl_defender.1 as f32);
            let factor_hlt: f32 = (hlt_attacker.0 as f32) / (hlt_attacker.1 as f32);
            let factor_org: f32 = (org_attacker as f32) / (PERCENT_100 as f32);

            factor_mrl + factor_hlt + factor_org + (dcy_weapon as f32)
        };
        let minus: u16 = {
            let factor_spl: f32 = (spl_defender.0 as f32) / (spl_defender.1 as f32);
            let factor_org: f32 = (org_defender as f32) / (PERCENT_100 as f32);
            let factor_prc: f32 = (prc_weapon + 1) as f32;
            let factor: f32 = (factor_spl + factor_org) / factor_prc;

            ((def_defender as f32) * factor) as u16
        };
        let damage_base: u16 = ((damage_weapon * factor) as u16).saturating_sub (minus);
        let damage_mrl: u16 = (damage_base * (prc_weapon + 1)) + (damage_magic * (dcy_weapon + 1));
        let damage_hlt: u16 = (damage_base * (slh_weapon + 1)) + damage_magic;
        let damage_spl: u16 = damage_base + damage_magic;
        let factor_defeat: u16 = if defender.is_retreat () {
            FACTOR_RETREAT
        } else if defender.is_rout () {
            FACTOR_ROUT
        } else {
            FACTOR_FIGHT
        };

        (damage_mrl, damage_hlt * factor_defeat, damage_spl * factor_defeat)
    }

    pub fn is_retreat (&self) -> bool {
        let mrl: u16 = self.get_statistic (MRL).0;

        mrl < THRESHOLD_RETREAT_MRL
    }

    pub fn is_rout (&self) -> bool {
        let mrl: u16 = self.get_statistic (MRL).0;

        mrl < THRESHOLD_ROUT_MRL
    }
}

impl Display for UnitStatistics {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        let mut display: String = String::from ("");

        for (i, statistic) in self.0.iter ().enumerate () {
            let value: u16 = match statistic {
                Capacity::Constant (c, _, _) => *c,
                Capacity::Quantity (c, _) => *c,
            };

            display.push_str (&format! ("({:?}: {}) ", UNIT_STATISTICS[i], value));
        }

        write! (f, "[ {}]", display)
    }
}

#[derive (Debug)]
pub struct Unit {
    id: ID,
    scene: Rc<Scene>,
    statistics: UnitStatistics,
    modifier_terrain_id: Option<ID>,
    modifiers: Vec<Modifier>,
    attribute_on_hit: Option<Attribute>,
    weapons: Vec<Weapon>,
    skill_passive_id: Option<ID>,
    skills: Vec<Skill>,
    magic_ids: Vec<ID>,
    weapon_active: usize,
    faction_id: ID,
    leader_id: Option<ID>,
    is_alive: bool,
}

impl Unit {
    #[allow (clippy::too_many_arguments)]
    pub fn new (id: ID, scene: Rc<Scene>, statistics: UnitStatistics, weapons: &[ID], skill_passive_id: Option<ID>, skill_ids: &[ID], magics_usable: &[bool; Element::Length as usize], faction_id: ID, leader_id: Option<ID>) -> Self {
        let modifier_terrain_id: Option<ID> = None;
        let modifiers: Vec<Modifier> = Vec::new ();
        let attribute_on_hit: Option<Attribute> = None;
        let weapons: Vec<Weapon> = weapons.iter ().map (|w: &ID| *scene.get_weapon (w)).collect ();
        let skills: Vec<Skill> = skill_ids.iter ().map (|s: &ID| *scene.get_skill (s)).collect ();
        let magic_ids: Vec<ID> = scene.magics_iter ().filter (|magic: &&Magic|
            magics_usable[magic.get_element () as usize] && statistics.get_statistic (MAG).0 >= magic.get_cost ()
        ).map (|magic: &Magic| magic.get_id ()).collect ();
        let weapon_active: usize = 0;
        let is_alive: bool = true;

        Self { id, scene, statistics, modifier_terrain_id, modifiers, attribute_on_hit, magic_ids, skill_passive_id, skills, weapons, weapon_active, faction_id, leader_id, is_alive }
    }

    pub fn get_statistic (&self, statistic: UnitStatistic) -> (u16, u16) {
        self.statistics.get_statistic (statistic)
    }

    pub fn set_statistic (&mut self, statistic: UnitStatistic, value: u16) {
        self.statistics.set_statistic (statistic, value);

        self.update_is_alive ();
    }

    fn change_statistic_flat (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) {
        self.statistics.change_statistic_flat (statistic, change, is_add);

        self.update_is_alive ();
    }

    fn change_statistic_percentage (&mut self, statistic: UnitStatistic, change: u16, is_add: bool) {
        self.statistics.change_statistic_percentage (statistic, change, is_add);

        self.update_is_alive ();
    }

    pub fn apply_inactive_skills (&mut self) {
        if let Some (s) = self.skill_passive_id {
            let skill_passive: &Skill = self.scene.get_skill (&s);
            let appliable_passive: Box<dyn Appliable> = skill_passive.try_yield_appliable (Rc::clone (&self.scene))
                    .unwrap_or_else (|| panic! ("Invalid skill {:?}", skill_passive));

            self.add_appliable (appliable_passive);
        }

        let appliables_toggle: Vec<Box<dyn Appliable>> = self.skills.iter ().filter_map (|skill: &Skill|
            if skill.is_toggled () {
                let appliable_passive: Box<dyn Appliable> = skill.try_yield_appliable (Rc::clone (&self.scene))
                        .unwrap_or_else (|| panic! ("Invalid skill {:?}", skill));

                Some (appliable_passive)
            } else {
                None
            }
        ).collect ();

        for appliable_toggle in appliables_toggle {
            self.add_appliable (appliable_toggle);
        }
    }

    pub fn change_modifier_terrain (&mut self, modifier_terrain_id: Option<ID>) {
        if let Some (modifier_terrain_id) = self.modifier_terrain_id {
            self.remove_appliable (AppliableKind::Modifier (modifier_terrain_id));
        }

        if let Some (modifier_terrain_id) = modifier_terrain_id {
            let modifier: Modifier = *self.scene.get_modifier (&modifier_terrain_id);
            let appliable: Box<dyn Appliable> = Box::new (modifier);

            self.modifier_terrain_id = Some (modifier_terrain_id);
            self.add_appliable (appliable);
        }
    }

    pub fn set_leader_id (&mut self, leader_id: ID) {
        self.leader_id = self.leader_id.map (|_| leader_id);
    }

    pub fn try_add_passive (&mut self, skill_id: &ID, distance: usize) -> bool {
        if self.leader_id.is_some () {
            let appliable: AppliableKind = self.scene.get_skill (skill_id).get_appliable ();
            let org: u16 = self.get_statistic (ORG).0;
            let factor: f32 = (org / PERCENT_100) as f32;
            let threshold: usize = ((THRESHOLD_SKILL_PASSIVE as f32) * factor) as usize;

            if distance > threshold {
                self.remove_appliable (appliable);
                self.skill_passive_id = None;

                false
            } else if self.skill_passive_id.is_none () {
                let appliable: Box<dyn Appliable> = appliable.appliable (Rc::clone (&self.scene));

                self.skill_passive_id = Some (*skill_id);

                self.add_appliable (appliable)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn is_retreat (&self) -> bool {
        self.statistics.is_retreat ()
    }

    pub fn is_rout (&self) -> bool {
        self.statistics.is_rout ()
    }

    pub fn is_alive (&self) -> bool {
        self.is_alive
    }

    fn update_is_alive (&mut self) -> bool {
        self.is_alive = self.get_statistic (HLT).0 > 0 && self.get_statistic (MRL).0 > 0;

        self.is_alive
    }

    pub fn start_turn (&mut self) {
        let mut modifiers_reapply: Vec<Box<dyn Appliable>> = Vec::new ();

        for modifier in &self.modifiers {
            if modifier.is_every_turn () {
                let modifier: Modifier = *modifier;
                let mut modifier: Box<Modifier> = Box::new (modifier);

                modifier.set_is_every_turn (false);
                modifiers_reapply.push (modifier);
            }
        }

        for appliable in modifiers_reapply {
            self.add_appliable (appliable);
        }

        self.update_is_alive ();
    }

    pub fn switch_weapon (&mut self) -> ID {
        self.weapon_active = (self.weapon_active + 1) % self.weapons.len ();

        self.weapon_active
    }

    pub fn act_attack (&mut self) -> (u16, &Weapon) {
        let org: u16 = self.get_statistic (ORG).0;
        let dividend: f32 = (org / PERCENT_100) as f32;
        let drain_spl: u16 = (DRAIN_SPL / dividend) as u16;

        self.change_statistic_flat (SPL, drain_spl, false);
        self.update_is_alive ();

        (self.get_statistic (MOV).0, self.get_weapon ())
    }

    pub fn take_damage (&mut self, damage_mrl: u16, damage_hlt: u16, damage_spl: u16) -> Option<Box<dyn Appliable>> {
        self.change_statistic_flat (MRL, damage_mrl, false);
        self.change_statistic_flat (HLT, damage_hlt, false);
        self.change_statistic_flat (SPL, damage_spl, false);
        self.update_is_alive ();

        self.try_yield_appliable (Rc::clone (&self.scene))
    }

    pub fn act_skill (&mut self, skill_id: &ID) -> (u16, &Skill) {
        let drain_spl: u16 = (DRAIN_SPL * FACTOR_SKILL) as u16;
        let index: usize = self.skills.iter ()
                .position (|s: &Skill| s.get_id () == *skill_id)
                .unwrap_or_else (|| panic! ("Skill {:?} not found", skill_id));
        let appliable_old: Option<AppliableKind> = {
            let skill: &mut Skill = &mut self.skills[index];

            if skill.is_toggled () {
                let (appliable_old, _): (AppliableKind, AppliableKind) = skill.switch_attribute ();

                Some (appliable_old)
            } else if skill.is_timed () {
                skill.start_cooldown ();

                None
            } else {
                unreachable! ()
            }
        };

        if let Some (appliable_old) = appliable_old {
            self.remove_appliable (appliable_old);
        }

        self.change_statistic_flat (SPL, drain_spl, false);
        self.update_is_alive ();

        (self.get_statistic (MOV).0, &self.skills[index])
    }

    pub fn act_magic (&mut self, magic_id: &ID) -> (u16, &Magic) {
        assert! (self.magic_ids.contains (magic_id));

        let mag: u16 = self.get_statistic (MAG).0;
        let cost: u16 = self.scene.get_magic (magic_id).get_cost ();
        let drain_hlt: u16 = u16::max ((cost * DRAIN_HLT).saturating_sub (mag), 1);
        let drain_org: u16 = u16::max ((cost * PERCENT_1).saturating_sub (mag / PERCENT_1), PERCENT_1);
        let drain_spl: u16 = (DRAIN_SPL * FACTOR_MAGIC) as u16;

        self.change_statistic_flat (HLT, drain_hlt, false);
        self.change_statistic_flat (ORG, drain_org, false);
        self.change_statistic_flat (SPL, drain_spl, false);
        self.update_is_alive ();

        (self.get_statistic (MOV).0, self.scene.get_magic (magic_id))
    }

    pub fn act_wait (&mut self) -> u16 {
        let drain_spl: u16 = (DRAIN_SPL * FACTOR_WAIT) as u16;

        self.change_statistic_flat (SPL, drain_spl, false);
        // self.update_is_dead (); // No change to HLT

        self.get_statistic (MOV).0
    }

    fn recover_supplies (&mut self, city_ids: &[ID]) {
        if !self.is_retreat () && !city_ids.is_empty () {
            let mut recover_hlt: u16 = 0;
            let mut recover_spl: u16 = 0;

            for city_id in city_ids {
                let change_hlt: u16 = self.scene.get_city (city_id).get_manpower ();
                let change_spl: u16 = self.scene.get_city (city_id).get_equipment ();

                recover_hlt += change_hlt;
                recover_spl += change_spl;
            }

            self.change_statistic_flat (HLT, recover_hlt, true);
            self.change_statistic_flat (SPL, recover_spl, true);
        }
    }

    pub fn end_turn (&mut self, city_ids: &[ID], appliable: Option<Box<dyn Appliable>>) {
        self.recover_supplies (city_ids);
        self.change_statistic_flat (MRL, RECOVER_MRL, true);
        self.decrement_durations ();

        if let Some (a) = appliable {
            self.add_appliable (a);
        }

        self.update_is_alive ();
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_statistics (&self) -> UnitStatistics {
        self.statistics
    }

    pub fn get_weapon (&self) -> &Weapon {
        &self.weapons[self.weapon_active]
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }

    pub fn get_skill_passive_id (&self) -> Option<ID> {
        self.skill_passive_id
    }

    pub fn get_skill_ids (&self) -> Vec<ID> {
        self.skills.iter ().map (|s: &Skill| s.get_id ()).collect ()
    }

    pub fn get_skill_ids_actionable (&self) -> Vec<ID> {
        self.skills.iter ().filter_map (|s: &Skill|
            if s.is_timed () {
                if s.try_yield_appliable (Rc::clone (&self.scene)).is_some () {
                    Some (s.get_id ())
                } else {
                    None
                }
            } else if s.is_toggled () {
                Some (s.get_id ())
            } else {
                None
            }
        ).collect ()
    }

    pub fn get_magic_ids (&self) -> &[ID] {
        &self.magic_ids
    }

    pub fn get_leader_id (&self) -> ID {
        self.leader_id.map_or (self.id, |leader_id: ID| leader_id)
    }
}

impl Applier for Unit {
    fn try_yield_appliable (&self, scene: Rc<Scene>) -> Option<Box<dyn Appliable>> {
        self.attribute_on_hit.and_then (|s: Attribute| s.try_yield_appliable (scene))
    }

    fn get_target (&self) -> Target {
        Target::Enemy
    }
}

impl Dynamic for Unit {
    fn add_appliable (&mut self, appliable: Box<dyn Appliable>) -> bool {
        let kind: AppliableKind = appliable.kind ();

        match kind {
            AppliableKind::Modifier ( .. ) => {
                let modifier: Modifier = appliable.modifier ();

                if modifier.can_stack_or_is_flat () || !self.modifiers.contains (&modifier) {
                    for adjustment in modifier.get_adjustments () {
                        if let StatisticKind::Unit (s) = adjustment.0 {
                            self.change_statistic_percentage (s, adjustment.1, adjustment.2);
                        }
                    }

                    self.modifiers.push (modifier);

                    true
                } else {
                    false
                }
            }
            AppliableKind::Effect ( .. ) => {
                let effect: Effect = appliable.effect ();

                for adjustment in effect.get_adjustments () {
                    if let StatisticKind::Unit (s) = adjustment.0 {
                        if effect.can_stack_or_is_flat () {
                            self.change_statistic_flat (s, adjustment.1, adjustment.2);
                        } else {
                            self.change_statistic_percentage (s, adjustment.1, adjustment.2);
                        }
                    }
                }

                true
            }
            AppliableKind::Attribute ( .. ) => {
                let attribute: Attribute = appliable.attribute ();
                let trigger: Trigger = attribute.get_trigger ();

                match trigger {
                    Trigger::OnAttack => {
                        let weapon: &mut Weapon = &mut self.weapons[self.weapon_active];

                        weapon.add_appliable (appliable);

                        true
                    }
                    Trigger::OnHit => if let Target::Enemy = attribute.get_target () {
                        self.attribute_on_hit = Some (attribute);

                        true
                    } else {
                        false
                    }
                    Trigger::OnOccupy => panic! ("Invalid trigger {:?}", trigger),
                }
            }
        }
    }

    fn remove_appliable (&mut self, appliable: AppliableKind) -> bool {
        match appliable {
            AppliableKind::Modifier (modifier_id) => {
                let index: Option<usize> = self.modifiers.iter ()
                        .position (|modifier: &Modifier| modifier.get_id () == modifier_id);

                if let Some (i) = index {
                    let modifier: Modifier = self.modifiers.swap_remove (i);

                    for adjustment in modifier.get_adjustments () {
                        if let StatisticKind::Unit (statistic) = adjustment.0 {
                            match statistic {
                                ATK | DEF | MAG | MOV | ORG => self.change_statistic_percentage (statistic, adjustment.1, !adjustment.2),
                                _ => (),
                            }
                        }
                    }

                    true
                } else {
                    false
                }
            }
            AppliableKind::Effect ( .. ) => unimplemented! (),
            AppliableKind::Attribute (attribute_id) => {
                if let Some (attribute) = self.attribute_on_hit {
                    if attribute.get_id () == attribute_id {
                        self.attribute_on_hit = None;

                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    fn decrement_durations (&mut self) {
        let mut modifiers_survived: Vec<Modifier> = Vec::new ();
        let mut modifiers_expired: Vec<Modifier> = Vec::new ();

        for modifier in self.modifiers.iter_mut () {
            if modifier.decrement_duration () {
                modifiers_survived.push (*modifier);
            } else {
                modifiers_expired.push (*modifier);
            }
        }

        self.modifiers = modifiers_survived;

        for modifier in modifiers_expired {
            if let Some (modifier_id_next) = modifier.get_next_id () {
                let modifier: Modifier =  *self.scene.get_modifier (&modifier_id_next);
                let modifier: Box<Modifier> = Box::new (modifier);

                self.add_appliable (modifier);
            }
        }

        if let Some (mut attribute) = self.attribute_on_hit {
            self.attribute_on_hit = if attribute.decrement_duration () {
                Some (attribute)
            } else {
                None
            };
        }

        for skill in self.skills.iter_mut () {
            skill.decrement_duration ();
        }

        for weapon in self.weapons.iter_mut () {
            weapon.decrement_durations ();
        }
    }
}

impl Display for Unit {
    fn fmt (&self, f: &mut Formatter<'_>) -> fmt::Result {
        write! (f, "{}: {}\n{:?}\n{:?}", self.id, self.statistics, self.modifiers, self.attribute_on_hit)
    }
}

#[derive (Debug)]
pub struct UnitBuilder {
    id: ID,
    statistics: UnitStatistics,
    weapon_ids: &'static [ID],
    skill_passive_id: Option<ID>,
    skill_ids: &'static [ID],
    magics_usable: [bool; Element::Length as usize],
    faction_id: ID,
    leader_id: Option<ID>,
}

impl UnitBuilder {
    #[allow (clippy::too_many_arguments)]
    pub const fn new (id: ID, statistics: UnitStatistics, weapon_ids: &'static [ID], skill_passive_id: Option<ID>, skill_ids: &'static [ID], magics_usable: [bool; Element::Length as usize], faction_id: ID, leader_id: Option<ID>) -> Self {
        Self { id, statistics, weapon_ids, skill_passive_id, skill_ids, magics_usable, faction_id, leader_id }
    }

    pub fn build (&self, scene: Rc<Scene>) -> Unit {
        Unit::new (self.id, scene, self.statistics, self.weapon_ids, self.skill_passive_id, self.skill_ids, &self.magics_usable, self.faction_id, self.leader_id)
    }

    pub fn get_id (&self) -> ID {
        self.id
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::tests::generate_scene;

    fn generate_units () -> (Unit, Unit, Unit) {
        let scene = generate_scene ();
        let unit_builder_0 = scene.get_unit_builder (&0);
        let unit_0 = unit_builder_0.build (Rc::clone (&scene));
        let unit_builder_1 = scene.get_unit_builder (&1);
        let unit_1 = unit_builder_1.build (Rc::clone (&scene));
        let unit_builder_2 = scene.get_unit_builder (&2);
        let unit_2 = unit_builder_2.build (Rc::clone (&scene));

        (unit_0, unit_1, unit_2)
    }

    fn generate_modifiers () -> (Box<Modifier>, Box<Modifier>) {
        let scene = generate_scene ();
        let modifier_3 = *scene.get_modifier (&3);
        let modifier_3 = Box::new (modifier_3);
        let modifier_4 = *scene.get_modifier (&4);
        let modifier_4 = Box::new (modifier_4);

        (modifier_3, modifier_4)
    }

    fn generate_effects () -> (Box<Effect>, Box<Effect>) {
        let scene = generate_scene ();
        let effect_0 = *scene.get_effect (&0);
        let effect_0 = Box::new (effect_0);
        let effect_1 = *scene.get_effect (&1);
        let effect_1 = Box::new (effect_1);

        (effect_0, effect_1)
    }

    fn generate_attributes () -> (Box<Attribute>, Box<Attribute>, Box<Attribute>) {
        let scene = generate_scene ();
        let attribute_0 = *scene.get_attribute (&0);
        let attribute_0 = Box::new (attribute_0);
        let attribute_1 = *scene.get_attribute (&1);
        let attribute_1 = Box::new (attribute_1);
        let attribute_5 = *scene.get_attribute (&5);
        let attribute_5 = Box::new (attribute_5);

        (attribute_0, attribute_1, attribute_5)
    }

    #[test]
    fn unit_change_statistic_flat () {
        let (mut unit_0, _, _) = generate_units ();

        // Test constant
        unit_0.change_statistic_flat (ATK, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (ATK).0, 25);
        unit_0.change_statistic_flat (ATK, 5, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (ATK).0, 20);
        unit_0.change_statistic_flat (ATK, ATK_MAX, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (ATK).0, ATK_MAX);
        unit_0.change_statistic_flat (ATK, ATK_MAX, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (ATK).0, 0);
        // Test quantity
        unit_0.change_statistic_flat (HLT, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (HLT).0, 990);
        unit_0.change_statistic_flat (HLT, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (HLT).0, 995);
        unit_0.change_statistic_flat (HLT, HLT_MAX, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (HLT).0, HLT_MAX);
        unit_0.change_statistic_flat (HLT, HLT_MAX, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (HLT).0, 0);
    }

    #[test]
    fn unit_change_statistic_percentage () {
        let (mut unit_0, _, _) = generate_units ();

        // Test constant
        unit_0.change_statistic_percentage (ATK, 10, true); // Test additive change
        assert_eq! (unit_0.get_statistic (ATK).0, 22);
        unit_0.change_statistic_percentage (ATK, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (ATK).0, 20);
        unit_0.change_statistic_percentage (ATK, 1000, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (ATK).0, ATK_MAX);
        unit_0.change_statistic_percentage (ATK, 1000, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (ATK).0, 0);
        // Test quantity
        unit_0.change_statistic_percentage (HLT, 10, false); // Test subtractive change
        assert_eq! (unit_0.get_statistic (HLT).0, 900);
        unit_0.change_statistic_percentage (HLT, 5, true); // Test additive change
        assert_eq! (unit_0.get_statistic (HLT).0, 950);
        unit_0.change_statistic_percentage (HLT, 1000, true); // Test maximum change
        assert_eq! (unit_0.get_statistic (HLT).0, HLT_MAX);
        unit_0.change_statistic_percentage (HLT, 1000, false); // Test minimum change
        assert_eq! (unit_0.get_statistic (HLT).0, 0);
    }

    #[test]
    fn unit_apply_inactive_skills () {
        let (mut unit_0, _, _) = generate_units ();

        unit_0.apply_inactive_skills ();
        assert_eq! (unit_0.modifiers.len (), 2);
    }

    #[test]
    fn unit_change_modifier_terrain () {
        let (mut unit_0, _, _) = generate_units ();

        // Test empty modifier
        unit_0.change_modifier_terrain (None);
        assert! (unit_0.modifier_terrain_id.is_none ());
        assert! (unit_0.modifiers.is_empty ());
        // Test non-empty modifier
        unit_0.change_modifier_terrain (Some (3));
        assert_eq! (unit_0.modifier_terrain_id.unwrap (), 3);
        assert_eq! (unit_0.modifiers.len (), 1);
    }

    #[test]
    fn unit_set_leader_id () {
        let (mut unit_0, mut unit_1, _) = generate_units ();

        // Test leader set
        unit_0.set_leader_id (1);
        assert_eq! (unit_0.leader_id, None);
        assert_eq! (unit_0.get_leader_id (), 0);
        // Test follower set
        unit_1.set_leader_id (0);
        assert_eq! (unit_1.leader_id, Some (0));
        assert_eq! (unit_1.get_leader_id (), 0);
    }

    #[test]
    fn unit_try_add_passive () {
        let (mut unit_0, mut unit_1, _) = generate_units ();
        let skill_passive_id = unit_0.skill_passive_id.unwrap ();

        // Test leader add
        assert! (!unit_0.try_add_passive (&skill_passive_id, 0));
        // Test near add
        assert! (unit_1.try_add_passive (&skill_passive_id, 1));
        assert_eq! (unit_1.skill_passive_id.unwrap (), skill_passive_id);
        assert_eq! (unit_1.modifiers.len (), 1);
        // Test far add
        assert! (!unit_1.try_add_passive (&skill_passive_id, 2));
        assert! (unit_1.skill_passive_id.is_none ());
        assert! (unit_1.modifiers.is_empty ());
    }

    #[test]
    fn unit_start_turn () {
        let scene = generate_scene ();
        let (mut unit_0, _, _) = generate_units ();
        let modifier_7 = *scene.get_modifier (&7);
        let modifier_7 = Box::new (modifier_7);
        let modifier_8 = *scene.get_modifier (&8);
        let modifier_8 = Box::new (modifier_8);

        // Test normal modifier
        unit_0.add_appliable (modifier_7);
        assert_eq! (unit_0.get_statistic (DEF).0, 18);
        unit_0.start_turn ();
        assert_eq! (unit_0.get_statistic (DEF).0, 18);
        // Test repeatable modifier
        unit_0.add_appliable (modifier_8);
        assert_eq! (unit_0.get_statistic (MAG).0, 18);
        unit_0.start_turn ();
        assert_eq! (unit_0.get_statistic (MAG).0, 16);
    }

    #[test]
    fn unit_act_attack () {
        let scene = generate_scene ();
        let (mut unit_0, _, _) = generate_units ();
        let attribute_6 = *scene.get_attribute (&6);
        let attribute_6 = Box::new (attribute_6);

        // Test normal attack
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        let response = unit_0.act_attack ();
        assert_eq! (response.0, 10);
        assert_eq! (response.1.get_id (), 0);
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert! (spl_0_0 > spl_0_1);
        // Test OnAttack attack
        unit_0.add_appliable (attribute_6);
        let response = unit_0.act_attack ();
        assert_eq! (response.0, 10);
        assert_eq! (response.1.get_id (), 0);
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert! (spl_0_1 > spl_0_2);
    }

    #[test]
    fn unit_take_damage () {
        let scene = generate_scene ();
        let (mut unit_0, _, mut unit_2) = generate_units ();
        let statistics_0 = unit_0.statistics;
        let statistics_2 = unit_2.statistics;
        let weapon = *unit_0.get_weapon ();
        let attribute_5 = *scene.get_attribute (&5);
        let attribute_5 = Box::new (attribute_5);

        // Test normal attack
        let (damage_mrl, damage_hlt, damage_spl) = UnitStatistics::calculate_damage (&statistics_0, &statistics_2, &weapon);
        let mrl_2_0 = unit_2.get_statistic (MRL).0;
        let hlt_2_0 = unit_2.get_statistic (HLT).0;
        let spl_2_0 = unit_2.get_statistic (SPL).0;
        assert! (unit_2.take_damage (damage_mrl, damage_hlt, damage_spl).is_none ());
        let mrl_2_1 = unit_2.get_statistic (MRL).0;
        let hlt_2_1 = unit_2.get_statistic (HLT).0;
        let spl_2_1 = unit_2.get_statistic (SPL).0;
        assert_eq! (mrl_2_0 - damage_mrl, mrl_2_1);
        assert_eq! (hlt_2_0 - damage_hlt, hlt_2_1);
        assert_eq! (spl_2_0 - damage_spl, spl_2_1);
        // Test OnHit attack
        unit_0.add_appliable (attribute_5);
        let weapon = *unit_2.get_weapon ();
        let (damage_mrl, damage_hlt, damage_spl) =
            UnitStatistics::calculate_damage (&statistics_2, &statistics_0, &weapon);
        let mrl_0_0 = unit_0.get_statistic (MRL).0;
        let hlt_0_0 = unit_0.get_statistic (HLT).0;
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        assert! (unit_0
            .take_damage (damage_mrl, damage_hlt, damage_spl)
            .is_some ());
        let mrl_0_1 = unit_0.get_statistic (MRL).0;
        let hlt_0_1 = unit_0.get_statistic (HLT).0;
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert_eq! (mrl_0_0 - damage_mrl, mrl_0_1);
        assert_eq! (hlt_0_0 - damage_hlt, hlt_0_1);
        assert_eq! (spl_0_0 - damage_spl, spl_0_1);
        // Test switch attack
        assert_eq! (unit_2.switch_weapon (), 1);
        let weapon = *unit_2.get_weapon ();
        let (damage_mrl, damage_hlt, damage_spl) =
            UnitStatistics::calculate_damage (&statistics_2, &statistics_0, &weapon);
        assert! (unit_0
            .take_damage (damage_mrl, damage_hlt, damage_spl)
            .is_some ());
        let mrl_0_2 = unit_0.get_statistic (MRL).0;
        let hlt_0_2 = unit_0.get_statistic (HLT).0;
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert_eq! (mrl_0_1 - damage_mrl, mrl_0_2);
        assert_eq! (hlt_0_1 - damage_hlt, hlt_0_2);
        assert_eq! (spl_0_1 - damage_spl, spl_0_2);
    }

    #[test]
    fn unit_act_skill () {
        let (mut unit_0, _, _) = generate_units ();

        // Test timed skill
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        let response = unit_0.act_skill (&0);
        assert_eq! (response.0, 10);
        assert_eq! (response.1.get_id (), 0);
        assert_eq! (unit_0.skills[0].get_duration (), 2);
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert! (spl_0_0 > spl_0_1);
        // Test toggled skill
        let response = unit_0.act_skill (&2);
        assert_eq! (response.0, 10);
        assert_eq! (response.1.get_id (), 2);
        assert_eq! (unit_0.skills[0].get_appliable (), AppliableKind::Modifier (6));
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert! (spl_0_1 > spl_0_2);
    }

    #[test]
    fn unit_act_magic () {
        let (mut unit_0, _, _) = generate_units ();

        let hlt_0_0 = unit_0.get_statistic (HLT).0;
        let org_0_0 = unit_0.get_statistic (ORG).0;
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        let response = unit_0.act_magic (&0);
        assert_eq! (response.0, 10);
        assert_eq! (response.1.get_appliable (), AppliableKind::Modifier (4));
        let hlt_0_1 = unit_0.get_statistic (HLT).0;
        let org_0_1 = unit_0.get_statistic (ORG).0;
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert! (hlt_0_0 > hlt_0_1);
        assert! (org_0_0 > org_0_1);
        assert! (spl_0_0 > spl_0_1);
    }

    #[test]
    fn unit_recover_supplies () {
        let (mut unit_0, _, _) = generate_units ();

        unit_0.change_statistic_flat (HLT, 500, false);
        unit_0.change_statistic_flat (SPL, 500, false);

        // Test encircled recover
        let hlt_0_0 = unit_0.get_statistic (HLT).0;
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        unit_0.recover_supplies (&[]);
        let hlt_0_1 = unit_0.get_statistic (HLT).0;
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert_eq! (hlt_0_0, hlt_0_1);
        assert_eq! (spl_0_0, spl_0_1);
        // Test normal recover
        unit_0.recover_supplies (&[0]);
        let hlt_0_2 = unit_0.get_statistic (HLT).0;
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert! (hlt_0_1 < hlt_0_2);
        assert! (spl_0_1 < spl_0_2);
    }

    #[test]
    fn unit_end_turn () {
        let scene = generate_scene ();
        let (mut unit_0, _, _) = generate_units ();
        let effect_0 = *scene.get_effect (&0);
        let effect_0 = Box::new (effect_0) as Box<dyn Appliable>;
        let effect_0 = Some (effect_0);

        unit_0.change_statistic_flat (MRL, 500, false);
        unit_0.change_statistic_flat (HLT, 500, false);
        unit_0.change_statistic_flat (SPL, 500, false);
        unit_0.skills[0].start_cooldown ();

        // Test encircled recover
        let mrl_0_0 = unit_0.get_statistic (MRL).0;
        let hlt_0_0 = unit_0.get_statistic (HLT).0;
        let spl_0_0 = unit_0.get_statistic (SPL).0;
        unit_0.end_turn (&[], effect_0);
        let mrl_0_1 = unit_0.get_statistic (MRL).0;
        let hlt_0_1 = unit_0.get_statistic (HLT).0;
        let spl_0_1 = unit_0.get_statistic (SPL).0;
        assert_eq! (mrl_0_0 + RECOVER_MRL, mrl_0_1);
        assert! (hlt_0_0 > hlt_0_1);
        assert_eq! (spl_0_0, spl_0_1);
        assert_eq! (unit_0.skills[0].get_duration (), 1);
        // Test normal recover
        unit_0.end_turn (&[1], None);
        let mrl_0_2 = unit_0.get_statistic (MRL).0;
        let hlt_0_2 = unit_0.get_statistic (HLT).0;
        let spl_0_2 = unit_0.get_statistic (SPL).0;
        assert_eq! (mrl_0_1 + RECOVER_MRL, mrl_0_2);
        assert! (hlt_0_1 < hlt_0_2);
        assert! (spl_0_1 < spl_0_2);
        assert_eq! (unit_0.skills[0].get_duration (), 0);
    }

    #[test]
    fn unit_get_skill_ids_actionable () {
        let scene = generate_scene ();
        let (unit_0, unit_1, _) = generate_units ();
        let unit_3 = scene.get_unit_builder (&3).build (Rc::clone (&scene));

        // Test empty skills
        assert! (unit_1.get_skill_ids_actionable ().is_empty ());
        // Test timed skills
        assert_eq! (unit_0.get_skill_ids_actionable ().len (), 2);
        assert_eq! (unit_3.get_skill_ids_actionable ().len (), 3);
    }

    #[test]
    fn unit_add_appliable () {
        let scene = generate_scene ();
        let (mut unit_0, mut unit_1, _) = generate_units ();
        let (modifier_3, modifier_4) = generate_modifiers ();
        let modifier_5 = *scene.get_modifier (&5);
        let modifier_5 = Box::new (modifier_5);
        let (effect_0, effect_1) = generate_effects ();
        let (_, _, attribute_5) = generate_attributes ();
        let attribute_6 = *scene.get_attribute (&6);
        let attribute_6 = Box::new (attribute_6);

        // Test additive modifier
        assert! (unit_0.add_appliable (modifier_3.clone ()));
        assert_eq! (unit_0.modifiers.len (), 1);
        assert_eq! (unit_0.get_statistic (ATK).0, 24);
        // Test subtractive modifier
        assert! (unit_0.add_appliable (modifier_4.clone ())); // Test multiple adjustments
        assert_eq! (unit_0.modifiers.len (), 2);
        assert_eq! (unit_0.get_statistic (ATK).0, 26);
        assert_eq! (unit_0.get_statistic (DEF).0, 18);
        // Test stacking modifier
        assert! (unit_0.add_appliable (modifier_3.clone ()));
        assert_eq! (unit_0.modifiers.len (), 3);
        assert_eq! (unit_0.get_statistic (ATK).0, 30);
        assert! (unit_0.add_appliable (modifier_3));
        assert_eq! (unit_0.modifiers.len (), 4);
        assert_eq! (unit_0.get_statistic (ATK).0, 34);
        // Test non-stacking modifier
        assert! (unit_0.add_appliable (modifier_5.clone ()));
        assert! (!unit_0.add_appliable (modifier_5));
        assert_eq! (unit_0.modifiers.len (), 5);
        assert_eq! (unit_0.get_statistic (ATK).0, 32);
        assert_eq! (unit_0.get_statistic (DEF).0, 18);

        // Test flat effect
        assert! (unit_1.add_appliable (effect_0));
        assert_eq! (unit_1.get_statistic (HLT).0, 998);
        // Test percentage effect
        assert! (unit_1.add_appliable (effect_1)); // Test multiple adjustments
        assert_eq! (unit_1.get_statistic (ATK).0, 21);
        assert_eq! (unit_1.get_statistic (DEF).0, 19);

        // Test applier attribute
        assert! (unit_0.add_appliable (attribute_5));
        assert! (unit_0.attribute_on_hit.is_some ());
        assert! (unit_0.try_yield_appliable (Rc::clone (&scene)).is_some ());
        // Test weapon attribute
        assert! (unit_0.add_appliable (attribute_6));
        assert! (unit_0.weapons[unit_0.weapon_active].try_yield_appliable (Rc::clone (&scene)).is_some ());
    }

    #[test]
    fn unit_remove_appliable () {
        let (mut unit_0, _, _) = generate_units ();
        let (modifier_3, _) = generate_modifiers ();
        let (_, _, attribute_5) = generate_attributes ();

        // Test empty remove
        assert! (!unit_0.remove_appliable (AppliableKind::Modifier (3)));
        assert! (unit_0.modifiers.is_empty ());
        // Test non-empty remove
        unit_0.add_appliable (modifier_3);
        assert! (unit_0.remove_appliable (AppliableKind::Modifier (3)));
        assert! (unit_0.modifiers.is_empty ());

        // Test empty remove
        assert! (!unit_0.remove_appliable (AppliableKind::Attribute (0)));
        assert! (unit_0.attribute_on_hit.is_none ());
        // Test non-empty remove
        unit_0.add_appliable (attribute_5);
        assert! (unit_0.remove_appliable (AppliableKind::Attribute (5)));
        assert! (unit_0.attribute_on_hit.is_none ());
    }

    #[test]
    fn unit_decrement_durations () {
        let scene = generate_scene ();
        let (mut unit_0, mut unit_1, _) = generate_units ();
        let (modifier_3, modifier_4) = generate_modifiers ();
        let modifier_6 = *scene.get_modifier (&6);
        let modifier_6 = Box::new (modifier_6);
        let (attribute_0, attribute_1, _) = generate_attributes ();

        // Test empty modifier
        unit_0.decrement_durations ();
        assert! (unit_0.modifiers.is_empty ());
        // Test timed modifier
        unit_0.add_appliable (modifier_3.clone ());
        unit_0.decrement_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        unit_0.decrement_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        unit_0.decrement_durations ();
        assert! (unit_0.modifiers.is_empty ());
        // Test permanent modifier
        unit_0.add_appliable (modifier_4);
        unit_0.decrement_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        unit_0.decrement_durations ();
        assert_eq! (unit_0.modifiers.len (), 1);
        // Test linked modifier
        unit_1.add_appliable (modifier_6);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.modifiers.len (), 1);
        assert_eq! (unit_1.modifiers[0].get_next_id ().unwrap (), 5);
        unit_1.decrement_durations ();
        assert_eq! (unit_1.modifiers.len (), 1);
        assert! (unit_1.modifiers[0].get_next_id ().is_none ());

        // Test empty attribute
        unit_1.decrement_durations ();
        assert! (unit_0.attribute_on_hit.is_none ());
        // Test timed attribute
        unit_1.add_appliable (attribute_1);
        unit_1.decrement_durations ();
        assert! (unit_1.attribute_on_hit.is_some ());
        unit_1.decrement_durations ();
        assert! (unit_1.attribute_on_hit.is_some ());
        unit_1.decrement_durations ();
        assert! (unit_1.attribute_on_hit.is_none ());
        // Test permanent attribute
        unit_1.add_appliable (attribute_0);
        unit_1.decrement_durations ();
        assert! (unit_1.attribute_on_hit.is_some ());
        unit_1.decrement_durations ();
        assert! (unit_1.attribute_on_hit.is_some ());
        unit_1.decrement_durations ();
        assert! (unit_1.attribute_on_hit.is_some ());
    }
}
