use std::rc::Rc;
use super::{Appliable, Effect, Modifier, ModifierBuilder};
use crate::Lists;
use crate::common::ID;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Change {
    Modifier (ID, bool), // modifier, is flat
    Effect (ID), // effect
}

impl Change {
    pub fn modifier (&self, lists: Rc<Lists>) -> Modifier {
        match self {
            Change::Modifier (m, s) => {
                let modifier_builder: &ModifierBuilder = lists.get_modifier_builder (m);

                modifier_builder.build (*s)
            }
            Change::Effect ( .. ) => unimplemented! (),
        }
    }

    pub fn effect (&self, lists: Rc<Lists>) -> Effect {
        match self {
            Change::Modifier ( .. ) => unimplemented! (),
            Change::Effect (e) => {
                *lists.get_effect (e)
            }
        }
    }

    pub fn appliable (&self, lists: Rc<Lists>) -> Box<dyn Appliable> {
        match self {
            Change::Modifier ( .. ) => {
                let modifier: Modifier = self.modifier (lists);
                let appliable: Box<dyn Appliable> = Box::new (modifier);

                appliable
            }
            Change::Effect ( .. ) => {
                let effect: Effect = self.effect (lists);
                let appliable: Box<dyn Appliable> = Box::new (effect);

                appliable
            }
        }
    }
}
