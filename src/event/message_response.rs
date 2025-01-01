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
    FactionAddMember (bool),
    GridAddStatus,
    GridTryYieldAppliable (Option<Change>), // OnOccupy status
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
            Response::FactionAddMember ( .. ) => EVENT_FACTION_ADD_MEMBER,
            Response::GridAddStatus => EVENT_GRID_ADD_STATUS,
            Response::GridTryYieldAppliable ( .. ) => EVENT_GRID_TRY_YIELD_APPLIABLE,
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
