use crate::character::{SkillKind, Element, FactionBuilder, Magic, Skill, UnitBuilder, UnitStatistics, Weapon};
use crate::character::UnitStatistic::{MRL, HLT, SPL, ATK, DEF, MAG, MOV, ORG};
use crate::common::{DURATION_PERMANENT, ID_UNINITIALISED, Target};
use crate::dynamic::{Attribute, Effect, Modifier, Trigger};
use crate::dynamic::AppliableKind::{Attribute as AppliableAttribute, Effect as AppliableEffect, Modifier as AppliableModifier};
use crate::dynamic::StatisticKind::{Tile, Unit};
use crate::map::{Area, City, Location, Terrain, TileBuilder};

pub const MODIFIERS: &[Modifier] = &[
    Modifier::new (0, &[
        (Tile (false), 1, true),
    ], 2, false, false, None), // terrain_cost_+1
    Modifier::new (1, &[
        (Tile (false), 1, false),
    ], DURATION_PERMANENT, false, false, None), // terrain_cost_-1
    Modifier::new (2, &[
        (Tile (true), 1, false),
    ], 1, false, false, Some (0)), // terrain_cost_=1
    Modifier::new (3, &[
        (Unit (ATK), 20, true),
    ], 2, true, false, None), // atk_+20
    Modifier::new (4, &[
        (Unit (ATK), 10, true),
        (Unit (DEF), 10, false),
    ], DURATION_PERMANENT, true, true, None), // atk_+10_def_-10
    Modifier::new (5, &[
        (Unit (ATK), 10, false),
    ], 1, false, false, None), // atk_-10
    Modifier::new (6, &[
        (Unit (HLT), 2, false),
    ], 1, false, false, Some (5)), // poison
    Modifier::new (7, &[
        (Unit (DEF), 10, false),
    ], 1, false, false, None), // def_-10
    Modifier::new (8, &[
        (Unit (MAG), 10, false),
    ], 1, true, true, None), // mag_-10
    Modifier::new (9, &[
        (Tile (true), 1, false),
    ], 1, false, false, None), // terrain_cost_=1
];
pub const EFFECTS: &[Effect] = &[
    Effect::new (0, &[
        (Unit (HLT), 2, false),
    ], true), // hlt_-2
    Effect::new (1, &[
        (Unit (ATK), 5, true),
        (Unit (DEF), 5, false),
    ], false), // atk_+5_def_-5
];
pub const ATTRIBUTES: &[Attribute] = &[
    Attribute::new (0, AppliableModifier
     (3), Trigger::OnHit, DURATION_PERMANENT), // atk_stack_up
    Attribute::new (1, AppliableModifier
     (5), Trigger::OnHit, 2), // atk_down
    Attribute::new (2, AppliableModifier
     (6), Trigger::OnOccupy, 2), // poison_2
    Attribute::new (3, AppliableModifier
     (1), Trigger::OnOccupy, DURATION_PERMANENT), // terrain_cost_down_permanent
    Attribute::new (4, AppliableModifier
     (6), Trigger::OnOccupy, 2), // poison_2
    Attribute::new (5, AppliableModifier
     (6), Trigger::OnHit, 2), // poison_2
    Attribute::new (6, AppliableModifier
     (6), Trigger::OnAttack, 2), // poison_2
    Attribute::new (7, AppliableModifier
     (6), Trigger::OnAttack, DURATION_PERMANENT), // poison_permanent
    Attribute::new (8, AppliableModifier
     (4), Trigger::OnHit, DURATION_PERMANENT), // atk_up_def_down
    Attribute::new (9, AppliableModifier
     (7), Trigger::OnHit, DURATION_PERMANENT), // def_down
    Attribute::new (10, AppliableModifier
     (8), Trigger::OnAttack, DURATION_PERMANENT), // mag_down
];
pub const TERRAINS: &[Terrain] = &[
    Terrain::new (None, 1), // passable_1
    Terrain::new (Some (3), 2), // passable_2
    Terrain::new (None, 0), // impassable

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
pub const WEAPONS: &[Weapon] = &[
    Weapon::new (0, [20, 1, 1, 0], Area::Single, 1), // single
    Weapon::new (1, [20, 0, 2, 0], Area::Path (0), 2), // path
    Weapon::new (2, [10, 1, 0, 1], Area::Radial (2), 2), // radial
];
pub const MAGICS: &[Magic] = &[
    Magic::new (0, AppliableModifier (4), Target::This, Area::Single, 0, 10, Element::Dark), // def_self
    Magic::new (1, AppliableModifier (3), Target::This, Area::Single, 0, 21, Element::Dark), // atk_self
    Magic::new (2, AppliableModifier (6), Target::This, Area::Single, 0, 10, Element::Matter), // poison_target_others
    Magic::new (3, AppliableAttribute (2), Target::Map, Area::Radial (2), 0, 10, Element::Light), // poison_map
];
pub const SKILLS: &[Skill] = &[
    Skill::new (0, &[AppliableModifier (6)], Target::This, Area::Single, 0, SkillKind::Timed (0, 2)),
    Skill::new (1, &[AppliableModifier (5)], Target::This, Area::Single, 0, SkillKind::Passive),
    Skill::new (2, &[AppliableModifier (3), AppliableModifier (5)], Target::This, Area::Radial (2), 0, SkillKind::Toggled (0)),
    Skill::new (3, &[AppliableModifier (0)], Target::This, Area::Radial (2), 0, SkillKind::Timed (1, 1)), // DO NOT USE
    Skill::new (4, &[AppliableModifier (4)], Target::Ally, Area::Single, 0, SkillKind::Timed (0, 2)),
    Skill::new (5, &[AppliableModifier (4)], Target::Allies, Area::Radial (2), 0, SkillKind::Timed (0, 2)),
    Skill::new (6, &[AppliableModifier (4)], Target::This, Area::Single, 0, SkillKind::Timed (0, 2)),
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
pub const UNIT_LOCATIONS: &[Option<Location>] = &[
    Some ((0, 0)),
    None,
    Some ((1, 0)),
    None,
    None,
];
