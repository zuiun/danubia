use crate::engine::common::{Area, Capacity, DURATION_PERMANENT, Information, Modifier, Statistic, Status, Target};
use crate::engine::map::{City, Terrain};
use crate::engine::character::{Faction, Magic, Skill, Unit, UnitStatisticsBuilder, Weapon};

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
        City::new (524, 108, 24), // Ilyvó
        City::new (41, 2, 14), // Kismarton
        City::new (23, 3, 5), // Újvidék
        City::new (65, 13, 6), // Temesvár
        City::new (88, 4, 21), // Telsze
        City::new (156, 27, 18), // Kluż-Arad
        City::new (32, 5, 2), // Stanisławów
        City::new (124, 18, 22), // Jawaryn
        // Dainava
        City::new (109, 20, 9), // Alytus
        City::new (37, 2, 8), // Rėzeknė
        City::new (136, 26, 11), // Kuresarė
        City::new (18, 1, 3), // Pėčas
        City::new (53, 3, 16), // Cešynas
        // Powiessern
        City::new (203, 35, 14), // Memel
        City::new (115, 19, 12), // Stolp
        City::new (60, 3, 21), // Carlstadt
        City::new (83,14, 11) // Gnesen
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
    use super::*;
    use crate::engine::character::UnitStatistic;

    pub const MODIFIERS: [Modifier; 5] = [
        Modifier::new (0, [
            (Some ((Statistic::Tile, 1, true))),
            None, None, None
        ], 2, true), // terrain_cost_up
        Modifier::new (1, [
            (Some ((Statistic::Tile, 1, false))),
            None, None, None
        ], DURATION_PERMANENT, true), // terrain_cost_down
        Modifier::new (2, [
            (Some ((Statistic::Tile, 1, false))),
            None, None, None
        ], 1, false), // terrain_cost_1
        Modifier::new (3, [
            (Some ((Statistic::Unit (UnitStatistic::ATK), 10, true))),
            None, None, None
        ], 2, true), // atk_up
        Modifier::new (4, [
            (Some ((Statistic::Unit (UnitStatistic::ATK), 10, false))),
            None, None, None
        ], DURATION_PERMANENT, false), // atk_up
    ];
    pub const STATUSES: [Status; 3] = [
        Status::new (0, 2, Target::All (false), None), // atk_up
        Status::new (1, DURATION_PERMANENT, Target::All (false), None), // poison
        Status::new (1, DURATION_PERMANENT, Target::All (false), None), // terrain_cost_down
    ];
    pub const TERRAINS: [Terrain; 3] = [
        Terrain::new (Vec::new (), 1), // passable_1
        Terrain::new (Vec::new (), 2), // passable_2
        Terrain::new (Vec::new (), 0) // impassable
    ];
    pub const CITIES: [City; 4] = [
        City::new (100, 1, 1),
        City::new (100, 2, 1), // recover_spl
        City::new (100, 1, 2), // recover_hlt
        City::new (100, 2, 2)
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

pub mod information {
    use super::Information;

    pub const CITIES: [Information; 17] = [
        Information::new ("Ilyvó", ["", "", ""], 0),
        Information::new ("Kismarton", ["", "", ""], 0),
        Information::new ("Újvidék", ["", "", ""], 0),
        Information::new ("Temesvár", ["", "", ""], 0),
        Information::new ("Telsze", ["", "", ""], 0),
        Information::new ("Kluż-Arad", ["", "", ""], 0),
        Information::new ("Stanisławów", ["", "", ""], 0),
        Information::new ("Jawaryn", ["", "", ""], 0),
        Information::new ("Alytus", ["", "", ""], 0),
        Information::new ("Rėzeknė", ["", "", ""], 0),
        Information::new ("Kuresarė", ["", "", ""], 0),
        Information::new ("Pėčas", ["", "", ""], 0),
        Information::new ("Cešynas", ["", "", ""], 0),
        Information::new ("Memel", ["", "", ""], 0),
        Information::new ("Stolp", ["", "", ""], 0),
        Information::new ("Carlstadt", ["", "", ""], 0),
        Information::new ("Gnesen", ["", "", ""], 0)
    ];
}
