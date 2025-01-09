use crate::character::{Activity, Element, FactionBuilder, Magic, Rank, Skill, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{Target, DURATION_PERMANENT, ID_UNINITIALISED};
use crate::dynamic::{Change, Effect, ModifierBuilder, Statistic, Status, Trigger};
use crate::map::{Area, City, Terrain, TileBuilder};

pub const MODIFIER_BUILDERS: &[ModifierBuilder] = &[

];
pub const EFFECTS: &[Effect] = &[

];
pub const STATUSES: &[Status] = &[

];
pub const TERRAINS: &[Terrain] = &[
    Terrain::new (None, 0), // Void
];
// TODO: recruits
pub const CITIES: &[City] = &[
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
    Weapon::new (0, [0, 2, 1, 0], Area::Single, 1), // Sabre
    Weapon::new (1, [0, 0, 3, 0], Area::Path (0), 2), // Lance
    Weapon::new (2, [0, 0, 2, 0], Area::Single, 2), // Pike
    Weapon::new (3, [0, 1, 2, 0], Area::Path (1), 1), // Glaive
    Weapon::new (4, [0, 1, 1, 1], Area::Single, 4), // Pistol
    Weapon::new (5, [0, 0, 2, 1], Area::Path (1), 8), // Musket
    Weapon::new (6, [0, 0, 3, 1], Area::Path (2), 12), // Rifle
    Weapon::new (7, [0, 0, 1, 0], Area::Single, 1), // Bayonet
    Weapon::new (8, [0, 0, 1, 2], Area::Radial (3), 12), // Mortar
];
pub const MAGICS: &[Magic] = &[

];
pub const SKILLS: &[Skill] = &[

];
pub const FACTION_BUILDERS: &[FactionBuilder] = &[

];
pub const UNIT_BUILDERS: &[UnitBuilder] = &[

];
pub const TILE_BUILDERS: &[&[TileBuilder]] = &[
    &[],
];
