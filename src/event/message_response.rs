use crate::character::UnitStatistics;
use crate::common::ID;
use crate::dynamic::Change;
use crate::map::{Location, Search};

// EVENT_observer_action
pub const EVENT_GAME_UNIT_DIE: ID = 0;
pub const EVENT_UNIT_TAKE_DAMAGE: ID = 1;
pub const EVENT_UNIT_ADD_STATUS: ID = 2;
pub const EVENT_UNIT_ADD_APPLIABLE: ID = 3;
pub const EVENT_GRID_FIND_UNITS: ID = 4;
pub const EVENT_GRID_FIND_LOCATIONS: ID = 5;
pub const EVENT_GRID_GET_UNIT_LOCATION: ID = 6;
pub const EVENT_GRID_IS_UNIT_ON_IMPASSABLE: ID = 7;
pub const EVENT_GRID_FIND_UNIT_CITIES: ID = 8;
pub const EVENT_UNIT_GET_STATISTICS: ID = 9;
pub const EVENT_FACTION_IS_MEMBER: ID = 10;
pub const EVENT_UNIT_GET_FACTION_ID: ID = 11;
pub const EVENT_FACTION_ADD_MEMBER: ID = 12;
pub const EVENT_GRID_ADD_STATUS: ID = 13;
pub const EVENT_GRID_TRY_YIELD_APPLIABLE: ID = 14;
pub const EVENT_UNIT_CHANGE_MODIFIER_TERRAIN: ID = 15;
pub const EVENT_GRID_FIND_DISTANCE_BETWEEN: ID = 16;
pub const EVENT_UNIT_TRY_ADD_PASSIVE: ID = 17;
pub const EVENT_reuse_later: ID = 18;
pub const EVENT_FACTION_ADD_FOLLOWER: ID = 19;
pub const EVENT_FACTION_GET_LEADER: ID = 20;
pub const EVENT_FACTION_GET_FOLLOWERS: ID = 21;
pub const EVENT_UNIT_SET_LEADER: ID = 22;
pub const EVENT_UNIT_SEND_PASSIVE: ID = 23;

pub const SUBJECT_UNIT_TYPE: ID = 0;
pub const SUBJECT_GRID_TYPE: ID = 1;
pub const SUBJECT_FACTION_TYPE: ID = 2;

const EVENTS_UNIT: [ID; 9] = [
    EVENT_UNIT_TAKE_DAMAGE,
    EVENT_UNIT_ADD_STATUS,
    EVENT_UNIT_ADD_APPLIABLE,
    EVENT_UNIT_GET_STATISTICS,
    EVENT_UNIT_GET_FACTION_ID,
    EVENT_UNIT_CHANGE_MODIFIER_TERRAIN,
    EVENT_UNIT_TRY_ADD_PASSIVE,
    EVENT_UNIT_SET_LEADER,
    EVENT_UNIT_SEND_PASSIVE,
];
const EVENTS_GRID: [ID; 8] = [
    EVENT_GRID_FIND_UNITS,
    EVENT_GRID_FIND_LOCATIONS,
    EVENT_GRID_GET_UNIT_LOCATION,
    EVENT_GRID_IS_UNIT_ON_IMPASSABLE,
    EVENT_GRID_FIND_UNIT_CITIES,
    EVENT_GRID_ADD_STATUS,
    EVENT_GRID_TRY_YIELD_APPLIABLE,
    EVENT_GRID_FIND_DISTANCE_BETWEEN,
];
const EVENTS_FACTION: [ID; 6] = [
    EVENT_FACTION_IS_MEMBER,
    EVENT_FACTION_ADD_MEMBER,
    EVENT_reuse_later,
    EVENT_FACTION_ADD_FOLLOWER,
    EVENT_FACTION_GET_LEADER,
    EVENT_FACTION_GET_FOLLOWERS,
];

pub fn event_iter (subject_type: ID) -> impl Iterator<Item = &'static ID> {
    match subject_type {
        SUBJECT_UNIT_TYPE => EVENTS_UNIT.iter (),
        SUBJECT_GRID_TYPE => EVENTS_GRID.iter (),
        SUBJECT_FACTION_TYPE => EVENTS_FACTION.iter (),
        _ => panic! ("Invalid subject {:?}", subject_type),
    }
}

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Message {
    GameUnitDie (ID), // unit
    UnitTakeDamage (ID, u16, u16, u16), // target, damage MRL, damage HLT, damage SPL
    UnitAddStatus (ID, ID), // target, status
    UnitAddAppliable (ID, Change), // target, appliable
    GridFindUnits (ID, Search),
    GridFindLocations (Location, Search),
    GridGetUnitLocation (ID), // unit
    GridIsUnitOnImpassable (ID), // unit
    GridFindUnitCities (ID, ID), // unit, daction
    UnitGetStatistics (ID), // unit
    FactionIsMember (ID, ID), // faction, unit
    UnitGetFactionId (ID), // unit
    FactionAddMember (ID, ID), // faction, unit
    GridAddStatus (Location, ID), // tile, status
    GridTryYieldAppliable (ID), // unit
    UnitChangeModifierTerrain (ID, ID), // unit, modifier
    GridFindDistanceBetween (ID, ID), // unit, unit
    UnitTryAddPassive (ID, ID), // unit, status
    reuse_later (ID, ID), // faction, unit
    FactionAddFollower (ID, ID, ID), // faction, follower, leader
    FactionGetLeader (ID, ID), // faction, unit
    FactionGetFollowers (ID, ID), // faction, unit
    UnitSetLeader (ID, ID), // unit, leader
    UnitSendPassive (ID), // unit
    TestAdd,
    TestSubtract,
}

impl Message {
    pub const fn discriminant (&self) -> ID {
        match self {
            Message::GameUnitDie ( .. ) => EVENT_GAME_UNIT_DIE,
            Message::UnitTakeDamage ( .. ) => EVENT_UNIT_TAKE_DAMAGE,
            Message::UnitAddStatus ( .. ) => EVENT_UNIT_ADD_STATUS,
            Message::UnitAddAppliable ( .. ) => EVENT_UNIT_ADD_APPLIABLE,
            Message::GridFindUnits ( .. ) => EVENT_GRID_FIND_UNITS,
            Message::GridFindLocations ( .. ) => EVENT_GRID_FIND_LOCATIONS,
            Message::GridGetUnitLocation ( .. ) => EVENT_GRID_GET_UNIT_LOCATION,
            Message::GridIsUnitOnImpassable ( .. ) => EVENT_GRID_IS_UNIT_ON_IMPASSABLE,
            Message::GridFindUnitCities ( .. ) => EVENT_GRID_FIND_UNIT_CITIES,
            Message::UnitGetStatistics ( .. ) => EVENT_UNIT_GET_STATISTICS,
            Message::FactionIsMember ( .. ) => EVENT_FACTION_IS_MEMBER,
            Message::UnitGetFactionId ( .. ) => EVENT_UNIT_GET_FACTION_ID,
            Message::FactionAddMember ( .. ) => EVENT_FACTION_ADD_MEMBER,
            Message::GridAddStatus ( .. ) => EVENT_GRID_ADD_STATUS,
            Message::GridTryYieldAppliable ( .. ) => EVENT_GRID_TRY_YIELD_APPLIABLE,
            Message::UnitChangeModifierTerrain ( .. ) => EVENT_UNIT_CHANGE_MODIFIER_TERRAIN,
            Message::GridFindDistanceBetween ( .. ) => EVENT_GRID_FIND_DISTANCE_BETWEEN,
            Message::UnitTryAddPassive ( .. ) => EVENT_UNIT_TRY_ADD_PASSIVE,
            Message::reuse_later ( .. ) => EVENT_reuse_later,
            Message::FactionAddFollower ( .. ) => EVENT_FACTION_ADD_FOLLOWER,
            Message::FactionGetLeader ( .. ) => EVENT_FACTION_GET_LEADER,
            Message::FactionGetFollowers ( .. ) => EVENT_FACTION_GET_FOLLOWERS,
            Message::UnitSetLeader ( .. ) => EVENT_UNIT_SET_LEADER,
            Message::UnitSendPassive ( .. ) => EVENT_UNIT_SEND_PASSIVE,
            Message::TestAdd => 0,
            Message::TestSubtract =>  1,
        }
    }
}

#[derive (Debug)]
#[derive (Clone)]
pub enum Response {
    GameUnitDie (ID),
    UnitTakeDamage (Option<Change>), // OnHit status
    UnitAddStatus,
    UnitAddAppliable,
    GridFindUnits (Vec<ID>),
    GridFindLocations (Vec<Location>),
    GridGetUnitLocation (Location),
    GridIsUnitOnImpassable (bool),
    GridFindUnitCities (Vec<ID>),
    UnitGetStatistics (UnitStatistics),
    FactionIsMember (bool),
    UnitGetFactionId (ID),
    FactionAddMember,
    GridAddStatus,
    GridTryYieldAppliable (Option<Change>), // OnOccupy status
    UnitChangeModifierTerrain,
    GridFindDistanceBetween (usize),
    UnitTryAddPassive,
    reuse_later,
    FactionAddFollower,
    FactionGetLeader (ID),
    FactionGetFollowers (Vec<ID>),
    UnitSetLeader,
    UnitSendPassive,
    TestAdd (u8),
    TestSubtract (u8),
}

impl Response {
    pub const fn discriminant (&self) -> ID {
        match self {
            Response::GameUnitDie ( .. ) => EVENT_GAME_UNIT_DIE,
            Response::UnitTakeDamage ( .. ) => EVENT_UNIT_TAKE_DAMAGE,
            Response::UnitAddStatus => EVENT_UNIT_ADD_STATUS,
            Response::UnitAddAppliable => EVENT_UNIT_ADD_APPLIABLE,
            Response::GridFindUnits ( .. ) => EVENT_GRID_FIND_UNITS,
            Response::GridFindLocations ( .. ) => EVENT_GRID_FIND_LOCATIONS,
            Response::GridGetUnitLocation ( .. ) => EVENT_GRID_GET_UNIT_LOCATION,
            Response::GridIsUnitOnImpassable ( .. ) => EVENT_GRID_IS_UNIT_ON_IMPASSABLE,
            Response::GridFindUnitCities ( .. ) => EVENT_GRID_FIND_UNIT_CITIES,
            Response::UnitGetStatistics ( .. ) => EVENT_UNIT_GET_STATISTICS,
            Response::FactionIsMember ( .. ) => EVENT_FACTION_IS_MEMBER,
            Response::UnitGetFactionId ( .. ) => EVENT_UNIT_GET_FACTION_ID,
            Response::FactionAddMember => EVENT_FACTION_ADD_MEMBER,
            Response::GridAddStatus => EVENT_GRID_ADD_STATUS,
            Response::GridTryYieldAppliable ( .. ) => EVENT_GRID_TRY_YIELD_APPLIABLE,
            Response::UnitChangeModifierTerrain => EVENT_UNIT_CHANGE_MODIFIER_TERRAIN,
            Response::GridFindDistanceBetween ( .. ) => EVENT_GRID_FIND_DISTANCE_BETWEEN,
            Response::UnitTryAddPassive => EVENT_UNIT_TRY_ADD_PASSIVE,
            Response::reuse_later => EVENT_reuse_later,
            Response::FactionAddFollower => EVENT_FACTION_ADD_FOLLOWER,
            Response::FactionGetLeader ( .. ) => EVENT_FACTION_GET_LEADER,
            Response::FactionGetFollowers ( .. ) => EVENT_FACTION_GET_FOLLOWERS,
            Response::UnitSetLeader => EVENT_UNIT_SET_LEADER,
            Response::UnitSendPassive => EVENT_UNIT_SEND_PASSIVE,
            Response::TestAdd ( .. ) => 0,
            Response::TestSubtract ( .. ) => 1,
        }
    }
}

impl PartialEq for Response {
    fn eq (&self, other: &Self) -> bool {
       self.discriminant () == other.discriminant ()
    }
}
