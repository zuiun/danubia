use crate::engine::common::Area;

type WeaponStatistics = [u8; WeaponStatistic::Length as usize];

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum WeaponStatistic {
    DMG, // damage - base damage
    SLH, // slash – modifier for physical damage, strong against manpower
    PRC, // pierce – modifier for physical damage, strong against morale
    DCY, // decay – modifier for magical damage
    Length
}

#[derive (Debug)]
pub struct Weapon {
    statistics: WeaponStatistics,
    area: Area,
    range: u8
}

impl Weapon {
    pub const fn new (statistics: WeaponStatistics, area: Area, range: u8) -> Self {
        Self { statistics, area, range }
    }

    pub fn get_statistic (&self, statistic: WeaponStatistic) -> u16 {
        self.statistics[statistic as usize] as u16
    }
}
