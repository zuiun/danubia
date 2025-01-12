use crate::character::{SkillKind, Element, FactionBuilder, Magic, Skill, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{DURATION_PERMANENT, ID_UNINITIALISED, Target};
use crate::dynamic::{AppliableKind, Effect, Modifier, StatisticKind, Status, Trigger};
use crate::map::{Area, City, Terrain, TileBuilder};

pub const MODIFIERS: &[Modifier] = &[
    Modifier::new (0, &[
        (StatisticKind::Tile (false), 1, true),
    ], 2, false), // terrain_cost_+1
    Modifier::new (1, &[
        (StatisticKind::Tile (false), 1, false),
    ], DURATION_PERMANENT, false), // terrain_cost_-1
    Modifier::new (2, &[
        (StatisticKind::Tile (true), 1, false),
    ], 1, false), // terrain_cost_=1
    Modifier::new (3, &[
        (StatisticKind::Unit (UnitStatistic::ATK), 20, true),
    ], 2, true), // atk_+20
    Modifier::new (4, &[
        (StatisticKind::Unit (UnitStatistic::ATK), 10, true),
        (StatisticKind::Unit (UnitStatistic::DEF), 10, false),
    ], DURATION_PERMANENT, true), // atk_+10_def_-10
    Modifier::new (5, &[
        (StatisticKind::Unit (UnitStatistic::ATK), 10, false),
    ], 1, false), // atk_-10
    Modifier::new (6, &[
        (StatisticKind::Unit (UnitStatistic::HLT), 2, false),
    ], 1, false), // poison
    Modifier::new (7, &[
        (StatisticKind::Unit (UnitStatistic::DEF), 10, false),
    ], 1, true), // def_-10
    Modifier::new (8, &[
        (StatisticKind::Unit (UnitStatistic::MAG), 10, false),
    ], 1, true), // mag_-10
];
pub const EFFECTS: &[Effect] = &[
    Effect::new (0, &[
        (StatisticKind::Unit (UnitStatistic::HLT), 2, false),
    ], true), // hlt_-2
    Effect::new (1, &[
        (StatisticKind::Unit (UnitStatistic::ATK), 5, true),
        (StatisticKind::Unit (UnitStatistic::DEF), 5, false),
    ], false), // atk_+5_def_-5
];
pub const STATUSES: &[Status] = &[
    Status::new (0, AppliableKind::Modifier (3), Trigger::None, DURATION_PERMANENT, Target::This, false, None), // atk_stack_up
    Status::new (1, AppliableKind::Modifier (5), Trigger::None, 2, Target::This, false, None), // atk_down
    Status::new (2, AppliableKind::Modifier (6), Trigger::OnOccupy, 2, Target::Map (ID_UNINITIALISED), false, None), // poison_2
    Status::new (3, AppliableKind::Modifier (1), Trigger::None, DURATION_PERMANENT, Target::Map (ID_UNINITIALISED), false, None), // terrain_cost_down_permanent
    Status::new (4, AppliableKind::Modifier (6), Trigger::OnOccupy, 2, Target::Map (ID_UNINITIALISED), false, Some (3)), // poison_2
    Status::new (5, AppliableKind::Modifier (6), Trigger::OnHit, 2, Target::Enemy, false, Some (0)), // poison_2
    Status::new (6, AppliableKind::Modifier (6), Trigger::OnAttack, 2, Target::Enemy, false, None), // poison_2
    Status::new (7, AppliableKind::Modifier (6), Trigger::OnAttack, DURATION_PERMANENT, Target::Enemy, false, None), // poison_permanent
    Status::new (8, AppliableKind::Modifier (4), Trigger::None, DURATION_PERMANENT, Target::This, true, None), // atk_up_def_down
    Status::new (9, AppliableKind::Modifier (7), Trigger::OnHit, DURATION_PERMANENT, Target::Enemy, true, None), // atk_up_def_down
    Status::new (10, AppliableKind::Modifier (8), Trigger::OnAttack, DURATION_PERMANENT, Target::Enemy, true, None), // atk_up_def_down
    Status::new (11, AppliableKind::Modifier (2), Trigger::None, 1, Target::Map (ID_UNINITIALISED), false, None), // atk_up_def_down
];
pub const TERRAINS: &[Terrain] = &[
    Terrain::new (None, 1), // DEBUG: passable_1
    Terrain::new (Some (3), 2), // DEBUG: passable_2
    Terrain::new (None, 0), // DEBUG: impassable

    Terrain::new (None, 0), // Void

];
pub const CITIES: &[City] = &[
    // DEBUG
    City::new (10, 1, 1, Some (1)),
    City::new (10, 2, 1, None),
    City::new (10, 1, 2, Some (3)),
    City::new (10, 2, 2, None),

    // Jassica
    City::new (524, 108, 24, None), // Ilyvó
    City::new (41, 2, 14, None), // Kismarton
    City::new (23, 3, 5, None), // Újvidék
    City::new (65, 13, 6, None), // Temesvár
    City::new (88, 4, 21, None), // Telsze
    City::new (156, 27, 18, None), // Kluż-Arad
    City::new (32, 5, 2, None), // Stanisławów
    City::new (124, 18, 22, None), // Jawaryn
    // Dainava
    City::new (109, 20, 9, None), // Alytus
    City::new (37, 2, 8, None), // Rėzeknė
    City::new (136, 26, 11, None), // Debrecenas
    City::new (18, 1, 3, None), // Pėčas
    City::new (53, 3, 16, None), // Cešynas
    // Powiessern
    City::new (203, 35, 14, None), // Memel
    City::new (115, 19, 12, None), // Stolp
    City::new (60, 3, 21, None), // Carlstadt
    City::new (83,14, 11, None), // Gnesen
];
// TODO: dmg, area, range
pub const WEAPONS: &[Weapon] = &[
    Weapon::new (0, [20, 1, 1, 0], Area::Single, 1), // DEBUG: single
    Weapon::new (1, [20, 0, 2, 0], Area::Path (0), 2), // DEBUG: path
    Weapon::new (2, [10, 1, 0, 1], Area::Radial (2), 2), // DEBUG: radial

    Weapon::new (3, [0, 2, 1, 0], Area::Single, 1), // Sabre
    Weapon::new (4, [0, 0, 3, 0], Area::Path (0), 2), // Lance
    Weapon::new (5, [0, 0, 2, 0], Area::Single, 2), // Pike
    Weapon::new (6, [0, 1, 2, 0], Area::Path (1), 1), // Glaive
    Weapon::new (7, [0, 1, 1, 1], Area::Single, 4), // Pistol
    Weapon::new (8, [0, 0, 2, 1], Area::Path (1), 8), // Musket
    Weapon::new (9, [0, 0, 3, 1], Area::Path (2), 12), // Rifle
    Weapon::new (10, [0, 0, 1, 0], Area::Single, 1), // Bayonet
    Weapon::new (11, [0, 0, 1, 2], Area::Radial (3), 12), // Mortar
];
pub const MAGICS: &[Magic] = &[
    Magic::new (0, 8, Target::This, Area::Single, 0, 10, Element::Dark), // def_self
    Magic::new (1, 0, Target::This, Area::Single, 0, 21, Element::Dark), // atk_self
    Magic::new (2, 6, Target::This, Area::Single, 0, 10, Element::Matter), // poison_target_others
    Magic::new (3, 2, Target::Map (ID_UNINITIALISED), Area::Radial (2), 0, 10, Element::Light), // poison_map
];
pub const SKILLS: &[Skill] = &[
    Skill::new (0, &[5], Target::This, Area::Single, 0, SkillKind::Timed (0, 2)),
    Skill::new (1, &[1], Target::This, Area::Single, 0, SkillKind::Passive),
    Skill::new (2, &[0, 1], Target::This, Area::Radial (2), 0, SkillKind::Toggled (0)),
    Skill::new (3, &[2], Target::This, Area::Radial (2), 0, SkillKind::Timed (1, 1)), // DO NOT USE
    Skill::new (4, &[8], Target::Ally, Area::Single, 0, SkillKind::Timed (0, 2)),
    Skill::new (5, &[8], Target::Allies, Area::Radial (2), 0, SkillKind::Timed (0, 2)),
    Skill::new (6, &[8], Target::This, Area::Single, 0, SkillKind::Timed (0, 2)),
];
pub const FACTION_BUILDERS: &[FactionBuilder] = &[
    FactionBuilder::new (0, &[2]),
    FactionBuilder::new (1, &[]),
    FactionBuilder::new (2, &[0]),
];
pub const UNIT_BUILDERS: &[UnitBuilder] = &[
    UnitBuilder::new (0,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[0], Some (1), &[0, 2, 3], [true, true, true], 0, None
    ),
    UnitBuilder::new (1,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[0], None, &[], [false, true, false], 0, Some (ID_UNINITIALISED)
    ),
    UnitBuilder::new (2,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[1, 2], Some (1), &[0, 2, 3], [false, false, false], 1, None
    ),
    UnitBuilder::new (3,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[0], Some (1), &[4, 5, 6], [false, false, false], 0, Some (ID_UNINITIALISED)
    ),
    UnitBuilder::new (4,
        UnitStatistics::new (1000, 1000, 1000, 20, 20, 20, 10, 1000),
        &[0], Some (1), &[0, 2, 3], [false, false, false], 2, None
    ),
];
pub const TILE_BUILDERS: &[&[TileBuilder]] = &[
    &[TileBuilder::new (0, 0, Some (0)), TileBuilder::new (0, 1, None), TileBuilder::new (0, 0, Some (1))],
    &[TileBuilder::new (1, 2, Some (2)), TileBuilder::new (1, 1, None), TileBuilder::new (2, 0, None)],
];
