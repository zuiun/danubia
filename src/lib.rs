pub mod character;
pub mod collections;
pub mod common;
pub mod controller;
pub mod dynamic;
pub mod event;
mod lists;
pub use self::lists::Scene;
pub use self::lists::debug;
pub use self::lists::information;
pub mod map;
pub mod system;

pub mod tests {
    use super::*;
    use std::rc::Rc;

    pub fn generate_scene () -> Rc<Scene> {
        Rc::new (Scene::default ())
    }
}
