use crate::character::{FactionBuilder, Magic, Skill, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{Capacity, DURATION_PERMANENT, Target};
use crate::dynamic::{Change, Effect, ModifierBuilder, StatisticType, Status, Trigger};
use crate::map::{Area, City, Terrain};

pub const MODIFIER_BUILDERS: [ModifierBuilder; 7] = [
    ModifierBuilder::new (0, [
        Some ((StatisticType::Tile (false), 1, true)),
        None, None, None
    ], 2), // terrain_cost_+1
    ModifierBuilder::new (1, [
        Some ((StatisticType::Tile (false), 1, false)),
        None, None, None
    ], DURATION_PERMANENT), // terrain_cost_-1
    ModifierBuilder::new (2, [
        Some ((StatisticType::Tile (true), 1, false)),
        None, None, None
    ], 1), // terrain_cost_=1
    ModifierBuilder::new (3, [
        Some ((StatisticType::Unit (UnitStatistic::ATK), 10, true)),
        None, None, None
    ], 2), // atk_+10
    ModifierBuilder::new (4, [
        Some ((StatisticType::Unit (UnitStatistic::ATK), 10, false)),
        Some ((StatisticType::Unit (UnitStatistic::DEF), 10, false)),
        None, None
    ], DURATION_PERMANENT), // atk_-10_def_-10
    ModifierBuilder::new (5, [
        Some ((StatisticType::Unit (UnitStatistic::ATK), 10, false)),
        None, None, None
    ], 1), // atk_-10
    ModifierBuilder::new (6, [
        Some ((StatisticType::Unit (UnitStatistic::HLT), 2, false)),
        None, None, None
    ], 1), // poison
];
pub const EFFECTS: [Effect; 2] = [
    Effect::new (0, [
        Some ((StatisticType::Unit (UnitStatistic::HLT), 2, false)),
        None, None, None
    ], true), // hlt_-2
    Effect::new (1, [
        Some ((StatisticType::Unit (UnitStatistic::ATK), 5, true)),
        Some ((StatisticType::Unit (UnitStatistic::DEF), 5, false)),
        None, None
    ], false), // atk_+5_def_-5
];
pub const STATUSES: [Status; 8] = [
    Status::new (Change::Modifier (3, true), Trigger::None, DURATION_PERMANENT, Target::This, None), // atk_stack_up
    Status::new (Change::Modifier (5, false), Trigger::None, 2, Target::This, None), // atk_down
    Status::new (Change::Modifier (6, false), Trigger::OnOccupy, 2, Target::Map, None), // poison_2
    Status::new (Change::Modifier (1, false), Trigger::None, DURATION_PERMANENT, Target::Map, None), // terrain_cost_down_permanent
    Status::new (Change::Modifier (6, false), Trigger::OnOccupy, 2, Target::Map, Some (3)), // poison_2
    Status::new (Change::Modifier (6, false), Trigger::OnHit, 2, Target::Enemy, Some (0)), // poison_2
    Status::new (Change::Modifier (6, false), Trigger::OnAttack, 2, Target::Enemy, None), // poison_2
    Status::new (Change::Modifier (6, false), Trigger::OnAttack, DURATION_PERMANENT, Target::Enemy, None), // poison_permanent
];
pub const TERRAINS: [Terrain; 3] = [
    Terrain::new (Vec::new (), 1), // passable_1
    Terrain::new (Vec::new (), 2), // passable_2
    Terrain::new (Vec::new (), 0), // impassable
];
pub const CITIES: [City; 4] = [
    City::new (100, 1, 1),
    City::new (100, 2, 1), // recover_spl
    City::new (100, 1, 2), // recover_hlt
    City::new (100, 2, 2),
];
pub const WEAPONS: [Weapon; 3] = [
    Weapon::new ([2, 1, 1, 0], Area::Single, 1), // single
    Weapon::new ([2, 0, 2, 0], Area::Path (0), 2), // path
    Weapon::new ([1, 1, 0, 1], Area::Radial (2), 2), // radial
];
pub const MAGICS: [Magic; 4] = [
    Magic::new (0, Target::Ally, Area::Radial (2), 0), // atk_others
    Magic::new (0, Target::This, Area::Single, 0), // atk_self
    Magic::new (6, Target::This, Area::Single, 0), // poison_target_others
    Magic::new (2, Target::Map, Area::Single, 0), // terrain_cost_down
];
pub const SKILLS: [Skill; 4] = [
    Skill::new ([5, 5], Target::This, Area::Single, 0, Capacity::Quantity (2, 2)),
    Skill::new ([0, 1], Target::Ally, Area::Single, 0, Capacity::Constant (1, 0, 0)),
    Skill::new ([0, 1], Target::Allies, Area::Radial (2), 0, Capacity::Constant (0, 1, 0)),
    Skill::new ([2, 2], Target::Map, Area::Radial (2), 0, Capacity::Constant (0, 1, 0)),
];
pub const FACTION_BUILDERS: [FactionBuilder; 3] = [
    FactionBuilder::new (0),
    FactionBuilder::new (1),
    FactionBuilder::new (2),
];
pub const UNIT_BUILDERS: [UnitBuilder; 5] = [
    UnitBuilder::new (0,
            UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
            [false, false, false], [0, 0], 0),
    UnitBuilder::new (1,
            UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
            [false, false, false], [0, 0], 0),
    UnitBuilder::new (2,
            UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
            [false, false, false], [1, 2], 1),
    UnitBuilder::new (3,
            UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
            [false, false, false], [0, 0], 0),
    UnitBuilder::new (4,
            UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
            [false, false, false], [0, 0], 2),
];
