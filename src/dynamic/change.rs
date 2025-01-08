use super::{Appliable, Effect, Modifier, ModifierBuilder};
use crate::common::ID;
use crate::Scene;
use std::rc::Rc;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Change {
    Modifier (ID, bool), // modifier, is flat
    Effect (ID), // effect
}

impl Change {
    pub fn modifier (&self, scene: Rc<Scene>) -> Modifier {
        match self {
            Change::Modifier (m, s) => {
                let modifier_builder: &ModifierBuilder = scene.get_modifier_builder (m);

                modifier_builder.build (*s)
            }
            Change::Effect ( .. ) => unimplemented! (),
        }
    }

    pub fn effect (&self, scene: Rc<Scene>) -> Effect {
        match self {
            Change::Modifier ( .. ) => unimplemented! (),
            Change::Effect (e) => {
                *scene.get_effect (e)
            }
        }
    }

    pub fn appliable (&self, scene: Rc<Scene>) -> Box<dyn Appliable> {
        match self {
            Change::Modifier ( .. ) => {
                let modifier: Modifier = self.modifier (scene);
                let appliable: Box<dyn Appliable> = Box::new (modifier);

                appliable
            }
            Change::Effect ( .. ) => {
                let effect: Effect = self.effect (scene);
                let appliable: Box<dyn Appliable> = Box::new (effect);

                appliable
            }
        }
    }
}
