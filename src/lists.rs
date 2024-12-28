use crate::engine::character::{Faction, Magic, Skill, Unit, UnitStatistic, UnitStatisticsBuilder, Weapon};
use crate::engine::common::{Area, Capacity, DURATION_PERMANENT, Target};
use crate::engine::dynamic::{Change, Effect, ModifierBuilder, Statistic, Status, Trigger};
use crate::engine::map::{City, Terrain};

pub mod game {
    use super::*;

    pub const MODIFIER_BUILDERS: [ModifierBuilder; 0] = [

    ];
    pub const EFFECTS: [Effect; 0] = [

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
        City::new (136, 26, 11), // Debrecenas
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
        Weapon::new ([0, 0, 3, 0], Area::Path (0), 2), // Lance
        Weapon::new ([0, 0, 2, 0], Area::Path (0), 2), // Pike
        Weapon::new ([0, 1, 2, 0], Area::Path (1), 1), // Glaive
        Weapon::new ([0, 1, 1, 1], Area::Path (1), 4), // Pistol
        Weapon::new ([0, 0, 2, 1], Area::Path (1), 8), // Musket
        Weapon::new ([0, 0, 3, 1], Area::Path (2), 12), // Rifle
        Weapon::new ([0, 0, 1, 0], Area::Single, 1), // Bayonet
        Weapon::new ([0, 0, 1, 2], Area::Radial (3), 12), // Mortar
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

    pub const MODIFIER_BUILDERS: [ModifierBuilder; 7] = [
        ModifierBuilder::new (0, [
            Some ((Statistic::Tile (false), 1, true)),
            None, None, None
        ]), // terrain_cost_+1
        ModifierBuilder::new (1, [
            Some ((Statistic::Tile (false), 1, false)),
            None, None, None
        ]), // terrain_cost_-1
        ModifierBuilder::new (2, [
            Some ((Statistic::Tile (true), 1, false)),
            None, None, None
        ]), // terrain_cost_=1
        ModifierBuilder::new (3, [
            Some ((Statistic::Unit (UnitStatistic::ATK), 10, true)),
            None, None, None
        ]), // atk_+10
        ModifierBuilder::new (4, [
            Some ((Statistic::Unit (UnitStatistic::ATK), 10, false)),
            Some ((Statistic::Unit (UnitStatistic::DEF), 10, false)),
            None, None
        ]), // atk_-10_def_-10
        ModifierBuilder::new (5, [
            Some ((Statistic::Unit (UnitStatistic::ATK), 10, false)),
            None, None, None
        ]), // atk_-10
        ModifierBuilder::new (5, [
            Some ((Statistic::Unit (UnitStatistic::HLT), 2, false)),
            None, None, None
        ]) // poison
    ];
    pub const EFFECTS: [Effect; 2] = [
        Effect::new (0, [
            Some ((Statistic::Unit (UnitStatistic::HLT), 2, false)),
            None, None, None
        ], true), // hlt_-2
        Effect::new (1, [
            Some ((Statistic::Unit (UnitStatistic::ATK), 5, true)),
            Some ((Statistic::Unit (UnitStatistic::DEF), 5, false)),
            None, None
        ], false) // atk_+5_def_-5
    ];
    pub const STATUSES: [Status; 8] = [
        Status::new (Change::Modifier (3, true), Trigger::None, DURATION_PERMANENT, Target::This, None), // atk_stack_up
        Status::new (Change::Modifier (5, false), Trigger::None, 2, Target::This, None), // atk_down
        Status::new (Change::Modifier (6, false), Trigger::OnOccupy, 2, Target::Map, None), // poison_2
        Status::new (Change::Modifier (1, false), Trigger::None, DURATION_PERMANENT, Target::Map, None), // terrain_cost_down_permanent
        Status::new (Change::Modifier (6, false), Trigger::OnOccupy, 2, Target::Map, Some (3)), // poison_2
        Status::new (Change::Modifier (6, false), Trigger::OnHit, 2, Target::Enemy, Some (0)), // poison_2
        Status::new (Change::Modifier (6, false), Trigger::OnAttack, 2, Target::Enemy, None), // poison_2
        Status::new (Change::Modifier (6, false), Trigger::OnAttack, DURATION_PERMANENT, Target::Enemy, None) // poison_permanent
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
        Magic::new (0, Target::Ally, Area::Radial (2), 0), // atk_others
        Magic::new (1, Target::This, Area::Single, 0), // atk_self
        Magic::new (1, Target::Enemy, Area::Single, 0), // poison_target_others
        Magic::new (2, Target::Map, Area::Single, 0) // terrain_cost_down
    ];
    pub const SKILLS: [Skill; 2] = [
        Skill::new ([0, 0, 0], Target::This, Area::Single, 0, Capacity::Quantity (2, 2)),
        Skill::new ([0, 0, 0], Target::This, Area::Single, 0, Capacity::Constant (1, 0, 0))
    ];
    // TODO: Factions and units are dynamic and probably can't be const
    // Get around this by making const builders, then populate them when constructing lists
    pub const FACTIONS: [Faction; 0] = [

    ];
    pub const UNITS: [Unit; 0] = [

    ];
}

pub mod information {
    use crate::engine::Information;

    pub const CITIES: [Information; 17] = [
        // Jassica
        Information::new ("Ilyvó", ["", "", ""], 0),
        Information::new ("Kismarton", ["", "", ""], 0),
        Information::new ("Újvidék", ["", "", ""], 0),
        Information::new ("Temesvár", ["", "", ""], 0),
        Information::new ("Telsze", ["", "", ""], 0),
        Information::new ("Kluż-Arad", ["", "", ""], 0),
        Information::new ("Stanisławów", ["", "", ""], 0),
        Information::new ("Jawaryn", ["", "", ""], 0),
        // Dainava
        Information::new ("Alytus", ["", "", ""], 0),
        Information::new ("Rėzeknė", ["", "", ""], 0),
        Information::new ("Debrecenas", ["", "", ""], 0),
        Information::new ("Pėčas", ["", "", ""], 0),
        Information::new ("Cešynas", ["", "", ""], 0),
        // Powiessern
        Information::new ("Memel", ["", "", ""], 0),
        Information::new ("Stolp", ["", "", ""], 0),
        Information::new ("Carlstadt", ["", "", ""], 0),
        Information::new ("Gnesen", ["", "", ""], 0)
    ];
}
