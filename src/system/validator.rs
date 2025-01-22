use self::Decision::*;
use super::Action;
use crate::map::Direction;
use sdl2::keyboard::Keycode;
use std::error::Error;

pub trait Validator<T> {
    fn validate (&mut self, input: Keycode) -> Result<Decision<T>, Box<dyn Error>>;
    fn get_prompt () -> &'static str;
}

#[derive (Debug)]
pub enum Decision<T> {
    Continue,
    Confirm (T),
    Cancel,
}

impl<T> Decision<T> {
    pub fn unwrap (self) -> T {
        match self {
            Continue => panic! ("Unwrap failed"),
            Confirm (val) => val,
            Cancel => panic! ("Unwrap failed"),
        }
    }

    pub fn is_continue (&self) -> bool {
        matches! (self, Continue)
    }

    pub fn is_confirm (&self) -> bool {
        matches! (self, Confirm ( .. ))
    }

    pub fn is_cancel (&self) -> bool {
        matches! (self, Cancel)
    }
}

#[derive (Debug)]
pub struct ActionValidator;

impl Validator<Action> for ActionValidator {
    fn validate (&mut self, input: Keycode) -> Result<Decision<Action>, Box<dyn Error>> {
        match input {
            Keycode::Q => Ok (Confirm (Action::Move)),
            Keycode::W => Ok (Confirm (Action::Weapon)),
            Keycode::A => Ok (Confirm (Action::Attack)),
            Keycode::S => Ok (Confirm (Action::Skill)),
            Keycode::D => Ok (Confirm (Action::Magic)),
            Keycode::Z => Ok (Confirm (Action::Wait)),
            Keycode::X => Ok (Cancel),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        "move (q), switch weapon (w), attack (a), skill (s), magic (d), wait (z), quit (x)"
    }
}

pub struct IndexValidator {
    index: usize,
    length: usize,
}

impl IndexValidator {
    pub fn new (length: usize) -> Self {
        let index: usize = 0;

        Self { index, length }
    }
}

impl Validator<usize> for IndexValidator {
    fn validate (&mut self, input: Keycode) -> Result<Decision<usize>, Box<dyn Error>> {
        match input {
            Keycode::A => {
                self.index = (self.index + 1) % self.length;

                Ok (Continue)
            }
            Keycode::D => {
                self.index = self.index.checked_sub (1).unwrap_or_else (|| self.length - 1);

                Ok (Continue)
            }
            Keycode::Z => Ok (Confirm (self.index)),
            Keycode::X => Ok (Cancel),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        "previous (a), next (d), confirm (z), cancel (x)"
    }
}

pub struct DirectionValidator;

impl Validator<Direction> for DirectionValidator {
    fn validate (&mut self, input: Keycode) -> Result<Decision<Direction>, Box<dyn Error>> {
        match input {
            Keycode::W => Ok (Confirm (Direction::Up)),
            Keycode::A => Ok (Confirm (Direction::Left)),
            Keycode::S => Ok (Confirm (Direction::Down)),
            Keycode::D => Ok (Confirm (Direction::Right)),
            Keycode::X => Ok (Cancel),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        "up (w), left (a), down (s), right (d), cancel (x)"
    }
}

pub struct MovementValidator;

impl Validator<Direction> for MovementValidator {
    fn validate (&mut self, input: Keycode) -> Result<Decision<Direction>, Box<dyn Error>> {
        match input {
            Keycode::W => Ok (Confirm (Direction::Up)),
            Keycode::A => Ok (Confirm (Direction::Left)),
            Keycode::S => Ok (Confirm (Direction::Down)),
            Keycode::D => Ok (Confirm (Direction::Right)),
            Keycode::Z => Ok (Confirm (Direction::Length)), // Placeholder direction
            Keycode::X => Ok (Cancel),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        "up (w), left (a), down (s), right (d), confirm (z), cancel (x)"
    }
}

pub struct ConfirmationValidator;

impl Validator<()> for ConfirmationValidator {
    fn validate (&mut self, input: Keycode) -> Result<Decision<()>, Box<dyn Error>> {
        match input {
            Keycode::Z => Ok (Confirm (())),
            Keycode::X => Ok (Cancel),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        "confirm (z), cancel (x)"
    }
}

#[cfg (test)]
mod tests {
    use super::*;

    #[test]
    fn action_validator_validate () {
        let mut validator = ActionValidator;

        assert! (matches! (validator.validate (Keycode::Q).unwrap ().unwrap (), Action::Move));
        assert! (matches! (validator.validate (Keycode::W).unwrap ().unwrap (), Action::Weapon));
        assert! (matches! (validator.validate (Keycode::A).unwrap ().unwrap (), Action::Attack));
        assert! (matches! (validator.validate (Keycode::S).unwrap ().unwrap (), Action::Skill));
        assert! (matches! (validator.validate (Keycode::D).unwrap ().unwrap (), Action::Magic));
        assert! (matches! (validator.validate (Keycode::Z).unwrap ().unwrap (), Action::Wait));
        assert! (validator.validate (Keycode::X).unwrap ().is_cancel ());
    }

    #[test]
    fn index_validator_validate () {
        let mut validator = IndexValidator::new (2);

        assert! (validator.validate (Keycode::A).unwrap ().is_continue ()); // 1
        assert! (validator.validate (Keycode::D).unwrap ().is_continue ()); // 0
        validator.validate (Keycode::A).unwrap (); // 1
        assert_eq! (validator.validate (Keycode::Z).unwrap ().unwrap (), 1);
        assert! (validator.validate (Keycode::X).unwrap ().is_cancel ());
    }

    #[test]
    fn direction_validator_validate () {
        let mut validator = DirectionValidator;

        // Test cancel validate
        // assert! (validator.validate ("x").unwrap ().is_none ());
        // Test normal validate
        assert! (matches! (validator.validate (Keycode::W).unwrap ().unwrap (), Direction::Up));
        assert! (matches! (validator.validate (Keycode::A).unwrap ().unwrap (), Direction::Left));
        assert! (matches! (validator.validate (Keycode::S).unwrap ().unwrap (), Direction::Down));
        assert! (matches! (validator.validate (Keycode::D).unwrap ().unwrap (), Direction::Right));
        assert! (validator.validate (Keycode::X).unwrap ().is_cancel ());
    }

    #[test]
    fn movement_validator_validate () {
        let mut validator = MovementValidator;

        assert! (matches! (validator.validate (Keycode::W).unwrap ().unwrap (), Direction::Up));
        assert! (matches! (validator.validate (Keycode::A).unwrap ().unwrap (), Direction::Left));
        assert! (matches! (validator.validate (Keycode::S).unwrap ().unwrap (), Direction::Down));
        assert! (matches! (validator.validate (Keycode::D).unwrap ().unwrap (), Direction::Right));
        assert! (matches! (validator.validate (Keycode::Z).unwrap ().unwrap (), Direction::Length));
        assert! (validator.validate (Keycode::X).unwrap ().is_cancel ());
    }

    #[test]
    fn confirmation_validator_validate () {
        let mut validator = ConfirmationValidator;

        assert! (validator.validate (Keycode::Z).unwrap ().is_confirm ());
        assert! (validator.validate (Keycode::X).unwrap ().is_cancel ());
    }
}
