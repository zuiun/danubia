use crate::engine::common::{Area, ID, Information, Target};

#[derive (Debug)]
pub struct Weapon {
    information: Information,
    area: Area,
    range: u8,
}

#[derive (Debug)]
pub struct Skill {
    information: Information,
    area: Area,
    range: u8,
    target: Target,
    is_passive: bool
}

#[derive (Debug)]
pub struct Character {
    information: Information,
    faction_id: ID,
    weapon_id: ID
}

impl Weapon {
    pub fn new (information: Information, area: Area, range: u8) -> Self {
        Self { information, area, range }
    }
}

impl Skill {

}

impl Character {
    pub fn new (information: Information, faction_id: ID, weapon_id: ID) -> Self {
        Self { information, faction_id, weapon_id }
    }

    pub fn get_faction_id (&self) -> ID {
        self.faction_id
    }
}

#[cfg (test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    fn generate_weapons () -> HashMap<ID, Weapon> {
        let sword = Weapon::new (Information::new (String::from ("Sword"), vec![String::from ("sword")], 0), Area::Single, 1);
        let spear = Weapon::new (Information::new (String::from ("Spear"), vec![String::from ("spear")], 0), Area::Path (1), 2);
        let book = Weapon::new (Information::new (String::from ("Book"), vec![String::from ("book")], 0), Area::Radial (2), 2);
        let mut weapons: HashMap<ID, Weapon> = HashMap::new ();

        weapons.insert (0, sword);
        weapons.insert (1, spear);
        weapons.insert (2, book);

        weapons
    }
}
