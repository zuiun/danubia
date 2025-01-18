pub mod character;
pub mod collections;
pub mod common;
pub mod controller;
pub mod dynamic;
pub mod event;
pub mod map;
pub mod system;

pub mod tests {    
    use super::*;
    use common::Scene;
    use std::rc::Rc;

    pub fn generate_scene () -> Rc<Scene> {
        Rc::new (Scene::default ())
    }
}
