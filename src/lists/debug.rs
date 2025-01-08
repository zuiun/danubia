use crate::character::{Activity, Element, FactionBuilder, Magic, Rank, Skill, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{Target, DURATION_PERMANENT, ID_UNINITIALISED};
use crate::dynamic::{Change, Effect, ModifierBuilder, Statistic, Status, Trigger};
use crate::map::{Area, City, Terrain, TileBuilder};

pub const MODIFIER_BUILDERS: &[ModifierBuilder] = &[
    ModifierBuilder::new (0, &[
        (Statistic::Tile (false), 1, true),
    ], 2), // terrain_cost_+1
    ModifierBuilder::new (1, &[
        (Statistic::Tile (false), 1, false),
    ], DURATION_PERMANENT), // terrain_cost_-1
    ModifierBuilder::new (2, &[
        (Statistic::Tile (true), 1, false),
    ], 1), // terrain_cost_=1
    ModifierBuilder::new (3, &[
        (Statistic::Unit (UnitStatistic::ATK), 20, true),
    ], 2), // atk_+20
    ModifierBuilder::new (4, &[
        (Statistic::Unit (UnitStatistic::ATK), 10, true),
        (Statistic::Unit (UnitStatistic::DEF), 10, false),
    ], DURATION_PERMANENT), // atk_+10_def_-10
    ModifierBuilder::new (5, &[
        (Statistic::Unit (UnitStatistic::ATK), 10, false),
    ], 1), // atk_-10
    ModifierBuilder::new (6, &[
        (Statistic::Unit (UnitStatistic::HLT), 2, false),
    ], 1), // poison
    ModifierBuilder::new (7, &[
        (Statistic::Unit (UnitStatistic::DEF), 10, false),
    ], 1), // def_-10
    ModifierBuilder::new (8, &[
        (Statistic::Unit (UnitStatistic::MAG), 10, false),
    ], 1), // mag_-10
];
pub const EFFECTS: &[Effect] = &[
    Effect::new (0, &[
        (Statistic::Unit (UnitStatistic::HLT), 2, false),
    ], true), // hlt_-2
    Effect::new (1, &[
        (Statistic::Unit (UnitStatistic::ATK), 5, true),
        (Statistic::Unit (UnitStatistic::DEF), 5, false),
    ], false), // atk_+5_def_-5
];
pub const STATUSES: &[Status] = &[
    Status::new (0, Change::Modifier (3, true), Trigger::None, DURATION_PERMANENT, Target::This, false, None), // atk_stack_up
    Status::new (1, Change::Modifier (5, false), Trigger::None, 2, Target::This, false, None), // atk_down
    Status::new (2, Change::Modifier (6, false), Trigger::OnOccupy, 2, Target::Map, false, None), // poison_2
    Status::new (3, Change::Modifier (1, false), Trigger::None, DURATION_PERMANENT, Target::Map, false, None), // terrain_cost_down_permanent
    Status::new (4, Change::Modifier (6, false), Trigger::OnOccupy, 2, Target::Map, false, Some (3)), // poison_2
    Status::new (5, Change::Modifier (6, false), Trigger::OnHit, 2, Target::Enemy, false, Some (0)), // poison_2
    Status::new (6, Change::Modifier (6, false), Trigger::OnAttack, 2, Target::Enemy, false, None), // poison_2
    Status::new (7, Change::Modifier (6, false), Trigger::OnAttack, DURATION_PERMANENT, Target::Enemy, false, None), // poison_permanent
    Status::new (8, Change::Modifier (4, true), Trigger::None, DURATION_PERMANENT, Target::This, true, None), // atk_up_def_down
    Status::new (9, Change::Modifier (7, true), Trigger::OnHit, DURATION_PERMANENT, Target::Enemy, true, None), // atk_up_def_down
    Status::new (10, Change::Modifier (8, true), Trigger::OnAttack, DURATION_PERMANENT, Target::Enemy, true, None), // atk_up_def_down
];
pub const TERRAINS: &[Terrain] = &[
    Terrain::new (None, 1), // passable_1
    Terrain::new (Some (3), 2), // passable_2
    Terrain::new (None, 0), // impassable
];
pub const CITIES: &[City] = &[
    City::new (100, 1, 1, Some (1)),
    City::new (100, 2, 1, None), // recover_spl
    City::new (100, 1, 2, None), // recover_hlt
    City::new (100, 2, 2, None),
];
pub const WEAPONS: &[Weapon] = &[
    Weapon::new ([2, 1, 1, 0], Area::Single, 1), // single
    Weapon::new ([2, 0, 2, 0], Area::Path (0), 2), // path
    Weapon::new ([1, 1, 0, 1], Area::Radial (2), 2), // radial
];
pub const MAGICS: &[Magic] = &[
    Magic::new (0, 8, Target::This, Area::Single, 0, 10, Element::Dark), // def_self
    Magic::new (1, 0, Target::This, Area::Single, 0, 21, Element::Dark), // atk_self
    Magic::new (2, 6, Target::This, Area::Single, 0, 10, Element::Matter), // poison_target_others
    Magic::new (3, 2, Target::Map, Area::Radial (2), 0, 10, Element::Light), // poison_map
];
pub const SKILLS: &[Skill] = &[
    Skill::new (0, 5, Target::This, Area::Single, 0, Activity::Timed (2, 2)),
    Skill::new (1, 1, Target::This, Area::Single, 0, Activity::Passive),
    Skill::new (2, 0, Target::This, Area::Radial (2), 0, Activity::Toggled (0, 1)),
    Skill::new (3, 2, Target::This, Area::Radial (2), 0, Activity::Timed (1, 1)), // DO NOT USE
    Skill::new (4, 8, Target::Ally, Area::Single, 0, Activity::Timed (2, 2)),
    Skill::new (5, 8, Target::Allies, Area::Radial (2), 0, Activity::Timed (2, 2)),
    Skill::new (6, 8, Target::This, Area::Single, 0, Activity::Timed (2, 2)),
];
pub const FACTION_BUILDERS: &[FactionBuilder] = &[
    FactionBuilder::new (0, &[2]),
    FactionBuilder::new (1, &[]),
    FactionBuilder::new (2, &[0]),
];
pub const UNIT_BUILDERS: &[UnitBuilder] = &[
    UnitBuilder::new (0,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[0], Some (1), &[0, 2, 3], [true, true, true], 0, Rank::Leader
    ),
    UnitBuilder::new (1,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[0], None, &[], [false, false, false], 0, Rank::Follower (ID_UNINITIALISED)
    ),
    UnitBuilder::new (2,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[1, 2], None, &[0, 2, 3], [false, false, false], 1, Rank::Leader
    ),
    UnitBuilder::new (3,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[0], None, &[4, 5, 6], [false, false, false], 0, Rank::Leader
    ),
    UnitBuilder::new (4,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[0], None, &[0, 2, 3], [false, false, false], 2, Rank::Leader
    ),
];
pub const TILE_BUILDERS: &[&[TileBuilder]] = &[
    &[TileBuilder::new (0, 0, Some (0)), TileBuilder::new (0, 1, None), TileBuilder::new (0, 0, Some (1))],
    &[TileBuilder::new (1, 2, Some (2)), TileBuilder::new (1, 1, None), TileBuilder::new (2, 0, None)],
];
