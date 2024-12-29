use crate::engine::character::UnitStatistics;
use crate::engine::common::{Area, ID};
use crate::engine::dynamic::Change;
use crate::engine::map::{Direction, Location};

// EVENT_observer_action
pub const EVENT_GAME_UNIT_DIE: ID = 0;
pub const EVENT_UNIT_TAKE_DAMAGE: ID = 1;
pub const EVENT_UNIT_ADD_STATUS: ID = 2;
pub const EVENT_UNIT_REUSE_LATER: ID = 3;
pub const EVENT_GRID_FIND_NEARBY_UNITS: ID = 4;
pub const EVENT_GRID_FIND_NEARBY_LOCATIONS: ID = 5;
pub const EVENT_GRID_GET_UNIT_LOCATION: ID = 6;
pub const EVENT_GRID_IS_UNIT_ON_IMPASSABLE: ID = 7;
pub const EVENT_GRID_FIND_UNIT_CITIES: ID = 8;
pub const EVENT_UNIT_GET_STATISTICS: ID = 9;
pub const EVENT_FACTION_IS_MEMBER: ID = 10;
pub const EVENT_UNIT_GET_FACTION_ID: ID = 11;
pub const EVENT_FACTION_ADD_MEMBER: ID = 12;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Message {
    GameUnitDie (ID), // unit
    UnitTakeDamage (ID, u16, u16), // target, damage_mrl, damage_hlt
    UnitAddStatus (ID, ID), // target, status
    Unit_REUSE_LATER (ID, ID),
    GridFindNearbyUnits (ID, Option<Direction>, Area, u8),
    GridFindNearbyLocations (Location, Option<Direction>, Area, u8),
    GridGetUnitLocation (ID), // unit
    GridIsUnitOnImpassable (ID), // unit
    GridFindUnitCities (ID, ID), // unit, daction
    UnitGetStatistics (ID), // unit
    FactionIsMember (ID, ID), // faction, unit
    UnitGetFactionId (ID), // unit
    FactionAddMember (ID, ID), // faction, unit
    TestAdd,
    TestSubtract,
}

impl Message {
    pub const fn discriminant (&self) -> ID {
        match self {
            Message::GameUnitDie (_) => EVENT_GAME_UNIT_DIE,
            Message::UnitTakeDamage (_, _, _) => EVENT_UNIT_TAKE_DAMAGE,
            Message::UnitAddStatus (_, _) => EVENT_UNIT_ADD_STATUS,
            Message::Unit_REUSE_LATER (_, _) => EVENT_UNIT_REUSE_LATER,
            Message::GridFindNearbyUnits (_, _, _, _) => EVENT_GRID_FIND_NEARBY_UNITS,
            Message::GridFindNearbyLocations (_, _, _, _) => EVENT_GRID_FIND_NEARBY_LOCATIONS,
            Message::GridGetUnitLocation (_) => EVENT_GRID_GET_UNIT_LOCATION,
            Message::GridIsUnitOnImpassable (_) => EVENT_GRID_IS_UNIT_ON_IMPASSABLE,
            Message::GridFindUnitCities (_, _) => EVENT_GRID_FIND_UNIT_CITIES,
            Message::UnitGetStatistics (_) => EVENT_UNIT_GET_STATISTICS,
            Message::FactionIsMember (_, _) => EVENT_FACTION_IS_MEMBER,
            Message::UnitGetFactionId (_) => EVENT_UNIT_GET_FACTION_ID,
            Message::FactionAddMember (_, _) => EVENT_FACTION_ADD_MEMBER,
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
    UnitAddStatus (ID, ID),
    Unit_REUSE_LATER (ID, ID),
    GridFindNearbyUnits (Vec<ID>),
    GridFindNearbyLocations (Vec<Location>),
    GridGetUnitLocation (Location),
    GridIsUnitOnImpassable (bool),
    GridFindUnitCities (Vec<ID>),
    UnitGetStatistics (UnitStatistics),
    FactionIsMember (bool),
    UnitGetFactionId (ID),
    FactionAddMember (bool),
    TestAdd (u8),
    TestSubtract (u8),
}

impl Response {
    pub const fn discriminant (&self) -> ID {
        match self {
            Response::GameUnitDie (_) => EVENT_GAME_UNIT_DIE,
            Response::UnitTakeDamage (_) => EVENT_UNIT_TAKE_DAMAGE,
            Response::UnitAddStatus (_, _) => EVENT_UNIT_ADD_STATUS,
            Response::Unit_REUSE_LATER (_, _) => EVENT_UNIT_REUSE_LATER,
            Response::GridFindNearbyUnits (_) => EVENT_GRID_FIND_NEARBY_UNITS,
            Response::GridFindNearbyLocations (_) => EVENT_GRID_FIND_NEARBY_LOCATIONS,
            Response::GridGetUnitLocation (_) => EVENT_GRID_GET_UNIT_LOCATION,
            Response::GridIsUnitOnImpassable (_) => EVENT_GRID_IS_UNIT_ON_IMPASSABLE,
            Response::GridFindUnitCities (_) => EVENT_GRID_FIND_UNIT_CITIES,
            Response::UnitGetStatistics (_) => EVENT_UNIT_GET_STATISTICS,
            Response::FactionIsMember (_) => EVENT_FACTION_IS_MEMBER,
            Response::UnitGetFactionId (_) => EVENT_UNIT_GET_FACTION_ID,
            Response::FactionAddMember (_) => EVENT_FACTION_ADD_MEMBER,
            Response::TestAdd (_) => 0,
            Response::TestSubtract (_) => 1,
        }
    }
}

impl PartialEq for Response {
    fn eq (&self, other: &Self) -> bool {
       self.discriminant () == other.discriminant ()
    }
}
