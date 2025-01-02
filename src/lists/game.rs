use crate::character::{FactionBuilder, Magic, Skill, UnitBuilder, UnitStatistic, UnitStatistics, Weapon};
use crate::common::{Capacity, DURATION_PERMANENT, Target};
use crate::dynamic::{Change, Effect, ModifierBuilder, StatisticType, Status, Trigger};
use crate::map::{Area, City, Terrain};

pub const MODIFIER_BUILDERS: [ModifierBuilder; 0] = [

];
pub const EFFECTS: [Effect; 0] = [

];
pub const STATUSES: [Status; 0] = [

];
pub const TERRAINS: [Terrain; 1] = [
    Terrain::new (Vec::new (), 0), // Void
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
    City::new (83,14, 11), // Gnesen
];
// TODO: dmg, area, range
pub const WEAPONS: [Weapon; 9] = [
    Weapon::new ([0, 2, 1, 0], Area::Single, 1), // Sabre
    Weapon::new ([0, 0, 3, 0], Area::Path (0), 2), // Lance
    Weapon::new ([0, 0, 2, 0], Area::Single, 2), // Pike
    Weapon::new ([0, 1, 2, 0], Area::Path (1), 1), // Glaive
    Weapon::new ([0, 1, 1, 1], Area::Single, 4), // Pistol
    Weapon::new ([0, 0, 2, 1], Area::Path (1), 8), // Musket
    Weapon::new ([0, 0, 3, 1], Area::Path (2), 12), // Rifle
    Weapon::new ([0, 0, 1, 0], Area::Single, 1), // Bayonet
    Weapon::new ([0, 0, 1, 2], Area::Radial (3), 12), // Mortar
];
pub const MAGICS: [Magic; 0] = [

];
pub const SKILLS: [Skill; 0] = [

];
pub const FACTION_BUILDERS: [FactionBuilder; 0] = [

];
pub const UNIT_BUILDERS: [UnitBuilder; 0] = [

];
