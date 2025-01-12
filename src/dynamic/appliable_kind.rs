use super::{Appliable, Effect, Modifier};
use crate::common::ID;
use crate::Scene;
use std::rc::Rc;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum AppliableKind {
    Modifier (ID), // modifier
    Effect (ID), // effect
}

impl AppliableKind {
    pub fn modifier (&self, scene: Rc<Scene>) -> Modifier {
        match self {
            AppliableKind::Modifier (m) => {
                *scene.get_modifier (m)
            }
            AppliableKind::Effect ( .. ) => unimplemented! (),
        }
    }

    pub fn effect (&self, scene: Rc<Scene>) -> Effect {
        match self {
            AppliableKind::Modifier ( .. ) => unimplemented! (),
            AppliableKind::Effect (e) => {
                *scene.get_effect (e)
            }
        }
    }

    pub fn appliable (&self, scene: Rc<Scene>) -> Box<dyn Appliable> {
        match self {
            AppliableKind::Modifier ( .. ) => {
                let modifier: Modifier = self.modifier (scene);
                let appliable: Box<dyn Appliable> = Box::new (modifier);

                appliable
            }
            AppliableKind::Effect ( .. ) => {
                let effect: Effect = self.effect (scene);
                let appliable: Box<dyn Appliable> = Box::new (effect);

                appliable
            }
        }
    }
}
