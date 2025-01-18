use super::{Appliable, Attribute, Effect, Modifier};
use crate::common::{ID, Scene};
use std::rc::Rc;

#[derive (Debug)]
#[derive (Clone, Copy)]
#[derive (PartialEq)]
pub enum AppliableKind {
    Modifier (ID), // modifier
    Effect (ID), // effect
    Attribute (ID), // attribute
}

impl AppliableKind {
    pub fn modifier (&self, scene: Rc<Scene>) -> Modifier {
        match self {
            AppliableKind::Modifier (m) => {
                *scene.get_modifier (m)
            }
            AppliableKind::Effect ( .. ) => unimplemented! (),
            AppliableKind::Attribute ( .. ) => unimplemented! (),
        }
    }

    pub fn modifier_id (&self) -> ID {
        match self {
            AppliableKind::Modifier (m) => {
                *m
            }
            AppliableKind::Effect ( .. ) => unimplemented! (),
            AppliableKind::Attribute ( .. ) => unimplemented! (),
        }
    }

    pub fn effect (&self, scene: Rc<Scene>) -> Effect {
        match self {
            AppliableKind::Modifier ( .. ) => unimplemented! (),
            AppliableKind::Effect (e) => {
                *scene.get_effect (e)
            }
            AppliableKind::Attribute ( .. ) => unimplemented! (),
        }
    }

    pub fn effect_id (&self) -> ID {
        match self {
            AppliableKind::Modifier ( .. ) => unimplemented! (),
            AppliableKind::Effect (e) => {
                *e
            }
            AppliableKind::Attribute ( .. ) => unimplemented! (),
        }
    }

    pub fn attribute (&self, scene: Rc<Scene>) -> Attribute {
        match self {
            AppliableKind::Modifier ( .. ) => unimplemented! (),
            AppliableKind::Effect ( .. ) => unimplemented! (),
            AppliableKind::Attribute ( a ) => {
                *scene.get_attribute (a)
            }
        }
    }

    pub fn attribute_id (&self) -> ID {
        match self {
            AppliableKind::Modifier ( .. ) => unimplemented! (),
            AppliableKind::Effect ( .. ) => unimplemented! (),
            AppliableKind::Attribute (a) => {
                *a
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
            AppliableKind::Attribute ( .. ) => {
                let attribute: Attribute = self.attribute (scene);
                let appliable: Box<dyn Appliable> = Box::new (attribute);

                appliable
            }
        }
    }
}
