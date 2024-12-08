use common::ID;

pub mod character;
pub mod common;
pub mod map;

// TODO: Anything that has an ID also has an Information mapped to it

pub struct Game {
    character_id: ID,
    faction_id: ID
}

impl Game {
    //
    pub fn update () -> () {
        
    }
}
