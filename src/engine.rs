pub mod common;
pub mod event;
pub mod map;
pub mod unit;

use common::ID;

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
