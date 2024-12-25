use crate::engine::common::{Area, Capacity, Modifier, Statistic, Status, Target, UnitStatistic};
use crate::engine::map::{City, Terrain};
use crate::engine::character::{Faction, Magic, Skill, Unit, Weapon};

pub mod game {
    use super::*;

    pub const MODIFIERS: [Modifier; 0] = [

    ];
    pub const STATUSES: [Status; 0] = [

    ];
    pub const TERRAINS: [Terrain; 1] = [
        Terrain::new (Vec::new (), 0) // Void
    ];
    pub const CITIES: [City; 17] = [
        // Jassica
        City::new (524, 0, 0), // Ilyvó
        City::new (41, 0, 0), // Bécs
        City::new (23, 0, 0), // Krakkó
        City::new (65, 0, 0), // Temesvár
        City::new (88, 0, 0), // Telsze
        City::new (156, 0, 0), // Kluż-Arad
        City::new (32, 0, 0), // Stanisławów
        City::new (124, 0, 0), // Jawaryn
        // Dainava
        City::new (109, 0, 0), // Alytus
        City::new (37, 0, 0), // Rėzeknė
        City::new (136, 0, 0), // Šauļi
        City::new (18, 0, 0), // Pēča
        City::new (53, 0, 0), // Cešina
        // Powiessern
        City::new (203, 0, 0), // Memel
        City::new (115, 0, 0), // Stolp
        City::new (58, 0, 0), // Carlstadt
        City::new (81, 0, 0) // Gnesen
    ];
    // TODO: dmg, area, range
    pub const WEAPONS: [Weapon; 9] = [
        Weapon::new ([0, 2, 1, 0], Area::Single, 1), // Sabre
        Weapon::new ([0, 0, 3, 0], Area::Single, 1), // Lance
        Weapon::new ([0, 0, 2, 0], Area::Single, 1), // Pike
        Weapon::new ([0, 1, 2, 0], Area::Single, 1), // Glaive
        Weapon::new ([0, 1, 1, 1], Area::Single, 1), // Pistol
        Weapon::new ([0, 0, 2, 1], Area::Single, 1), // Musket
        Weapon::new ([0, 0, 3, 1], Area::Single, 1), // Rifle
        Weapon::new ([0, 0, 1, 0], Area::Single, 1), // Bayonet
        Weapon::new ([0, 0, 1, 2], Area::Single, 1), // Mortar
    ];
    pub const MAGICS: [Magic; 0] = [

    ];
    pub const SKILLS: [Skill; 0] = [

    ];
    pub const FACTIONS: [Faction; 0] = [

    ];
    pub const UNITS: [Unit; 0] = [

    ];
}

pub mod debug {
    use crate::engine::common::UnitStatistic;

    use super::*;

    pub const MODIFIERS: [Modifier; 3] = [
        Modifier::new (0, [
            (Some ((Statistic::Unit (UnitStatistic::ATK), 10, true))),
            None, None, None
        ], Capacity::Quantity (5, 5), false),
        Modifier::new (1, [
            (Some ((Statistic::Unit (UnitStatistic::HLT), 5, false))),
            None, None, None
        ], Capacity::Quantity (5, 5), true),
        Modifier::new (2, [
            (Some ((Statistic::Tile, 1, false))),
            None, None, None
        ], Capacity::Quantity (5, 5), true)
    ];
    pub const STATUSES: [Status; 3] = [
        Status::new (0, Capacity::Constant (0, 0, 0), Target::All (false), None), // atk_up
        Status::new (1, Capacity::Constant (0, 0, 0), Target::All (false), None), // poison
        Status::new (2, Capacity::Constant (0, 0, 0), Target::All (false), None), // terrain_cost_down
    ];
    pub const TERRAINS: [Terrain; 3] = [
        Terrain::new (Vec::new (), 1), // passable_1
        Terrain::new (Vec::new (), 2), // passable_2
        Terrain::new (Vec::new (), 0) // impassable
    ];
    pub const CITIES: [City; 4] = [
        City::new (10, 0, 0), // recover_none
        City::new (10, 1, 0), // recover_spl
        City::new (10, 0, 1), // recover_hlt
        City::new (10, 1, 1) // recover_all
    ];
    pub const WEAPONS: [Weapon; 3] = [
        Weapon::new ([2, 1, 1, 0], Area::Single, 1), // single
        Weapon::new ([2, 0, 2, 0], Area::Path (0), 2), // path
        Weapon::new ([1, 1, 0, 1], Area::Radial (2), 2) // radial
    ];
    pub const MAGICS: [Magic; 4] = [
        Magic::new (0, Target::All (true), Area::Single, 0), // atk_others
        Magic::new (0, Target::Ally (true), Area::Single, 0), // atk_self
        Magic::new (1, Target::Ally (true), Area::Single, 0), // poison_target_others
        Magic::new (2, Target::Map, Area::Single, 0) // terrain_cost_down
    ];
    pub const SKILLS: [Skill; 0] = [

    ];
    // TODO: Factions and units are dynamic and probably can't be const
    // Get around this by making const builders, then populate them when constructing lists
    pub const FACTIONS: [Faction; 0] = [

    ];
    pub const UNITS: [Unit; 0] = [

    ];
}
