pub mod common;
pub mod controller;
pub mod collections;
pub mod dynamic;
pub mod event;
pub mod map;
pub mod character;
pub mod system;

mod lists;
pub use self::lists::Lists;
pub use self::lists::game;
pub use self::lists::debug;
pub use self::lists::information;

use std::rc::Rc;

// TODO: Anything that has an ID also has an Information mapped to it

pub mod tests {
    use super::*;

    pub fn generate_lists () -> Rc<Lists> {
        Rc::new (Lists::debug ())
    }
}
