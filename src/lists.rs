use crate::engine::common::Area;
use crate::engine::map::{City, Terrain};
use crate::engine::unit::{Faction, Magic, Skill, Unit, Weapon};

pub mod game {
    use super::*;

    pub const TERRAINS: [Terrain; 3] = [
        Terrain::new (Vec::new (), 1), // Grass
        Terrain::new (Vec::new (), 2), // Grass
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
    pub const WEAPONS: [Weapon; 0] = [

    ];
    pub const MAGICS: [Magic; 0] = [
        // 
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
        Weapon::new ([1, 1, 0], 2, Area::Single, 1), // sword
        Weapon::new ([0, 2, 0], 2, Area::Path (1), 2), // spear
        Weapon::new ([1, 0, 1], 1, Area::Radial (2), 2) // book
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
