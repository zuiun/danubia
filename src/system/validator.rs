use super::Action;
use crate::map::Direction;
use sdl2::keyboard::Keycode;
use std::error::Error;
use std::ops::ControlFlow::{self, Break, Continue};

pub trait Validator<B, C> {
    fn validate (&mut self, input: Keycode) -> Result<ControlFlow<B, C>, Box<dyn Error>>;
    fn get_prompt () -> &'static str;
}

#[derive (Debug)]
pub struct Unrepeatable;

#[derive (Debug)]
pub struct ActionValidator;

impl Validator<Option<Action>, Unrepeatable> for ActionValidator {
    fn validate (&mut self, input: Keycode) -> Result<ControlFlow<Option<Action>, Unrepeatable>, Box<dyn Error>> {
        match input {
            Keycode::Q => Ok (Break (Some (Action::Move))),
            Keycode::W => Ok (Break (Some (Action::Weapon))),
            Keycode::A => Ok (Break (Some (Action::Attack))),
            Keycode::S => Ok (Break (Some (Action::Skill))),
            Keycode::D => Ok (Break (Some (Action::Magic))),
            Keycode::Z => Ok (Break (Some (Action::Wait))),
            Keycode::X => Ok (Break (None)),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        // "move (q), switch weapon (w), attack (a), skill (s), magic (d), wait (z), quit (x)"
        "move (q), switch weapon (w), attack (a), skill (s), magic (d), wait (z)"
    }
}

pub struct IndexValidator {
    index: usize,
    length: usize,
}

impl IndexValidator {
    pub fn new (index: usize, length: usize) -> Self {
        Self { index, length }
    }
}

impl Validator<Option<usize>, usize> for IndexValidator {
    fn validate (&mut self, input: Keycode) -> Result<ControlFlow<Option<usize>, usize>, Box<dyn Error>> {
        match input {
            Keycode::A => {
                self.index = self.index.checked_sub (1).unwrap_or_else (|| self.length.saturating_sub (1));

                Ok (Continue (self.index))
            }
            Keycode::D => {
                self.index = (self.index + 1) % self.length;

                Ok (Continue (self.index))
            }
            Keycode::Z => Ok (Break (Some (self.index))),
            Keycode::X => Ok (Break (None)),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        "previous (a), next (d), confirm (z), cancel (x)"
    }
}

pub struct DirectionValidator;

impl Validator<Option<Direction>, Unrepeatable> for DirectionValidator {
    fn validate (&mut self, input: Keycode) -> Result<ControlFlow<Option<Direction>, Unrepeatable>, Box<dyn Error>> {
        match input {
            Keycode::W => Ok (Break (Some (Direction::Up))),
            Keycode::A => Ok (Break (Some (Direction::Left))),
            Keycode::S => Ok (Break (Some (Direction::Down))),
            Keycode::D => Ok (Break (Some (Direction::Right))),
            Keycode::X => Ok (Break (None)),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        "up (w), left (a), down (s), right (d), cancel (x)"
    }
}

pub struct MovementValidator;

impl Validator<bool, Direction> for MovementValidator {
    fn validate (&mut self, input: Keycode) -> Result<ControlFlow<bool, Direction>, Box<dyn Error>> {
        match input {
            Keycode::W => Ok (Continue (Direction::Up)),
            Keycode::A => Ok (Continue (Direction::Left)),
            Keycode::S => Ok (Continue (Direction::Down)),
            Keycode::D => Ok (Continue (Direction::Right)),
            Keycode::Z => Ok (Break (true)),
            Keycode::X => Ok (Break (false)),
            _ => Err (Box::from (String::from ("Invalid input"))),
        }
    }

    fn get_prompt () -> &'static str {
        "up (w), left (a), down (s), right (d), confirm (z), cancel (x)"
    }
}

pub struct ConfirmationValidator;

impl Validator<bool, Unrepeatable> for ConfirmationValidator {
    fn validate (&mut self, input: Keycode) -> Result<ControlFlow<bool, Unrepeatable>, Box<dyn Error>> {
        match input {
            Keycode::Z => Ok (Break (true)),
            Keycode::X => Ok (Break (false)),
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

        assert! (matches! (validator.validate (Keycode::Q).unwrap ().break_value ().unwrap ().unwrap (), Action::Move));
        assert! (matches! (validator.validate (Keycode::W).unwrap ().break_value ().unwrap ().unwrap (), Action::Weapon));
        assert! (matches! (validator.validate (Keycode::A).unwrap ().break_value ().unwrap ().unwrap (), Action::Attack));
        assert! (matches! (validator.validate (Keycode::S).unwrap ().break_value ().unwrap ().unwrap (), Action::Skill));
        assert! (matches! (validator.validate (Keycode::D).unwrap ().break_value ().unwrap ().unwrap (), Action::Magic));
        assert! (matches! (validator.validate (Keycode::Z).unwrap ().break_value ().unwrap ().unwrap (), Action::Wait));
        assert! (validator.validate (Keycode::X).unwrap ().break_value ().unwrap ().is_none ());
    }

    #[test]
    fn index_validator_validate () {
        let mut validator = IndexValidator::new (0, 2);

        assert_eq! (validator.validate (Keycode::A).unwrap ().continue_value ().unwrap (), 1); // 1
        assert_eq! (validator.validate (Keycode::D).unwrap ().continue_value ().unwrap (), 0); // 0
        validator.validate (Keycode::A).unwrap (); // 1
        assert_eq! (validator.validate (Keycode::Z).unwrap ().break_value ().unwrap ().unwrap (), 1);
        assert! (validator.validate (Keycode::X).unwrap ().break_value ().unwrap ().is_none ());
    }

    #[test]
    fn direction_validator_validate () {
        let mut validator = DirectionValidator;

        assert! (matches! (validator.validate (Keycode::W).unwrap ().break_value ().unwrap ().unwrap (), Direction::Up));
        assert! (matches! (validator.validate (Keycode::A).unwrap ().break_value ().unwrap ().unwrap (), Direction::Left));
        assert! (matches! (validator.validate (Keycode::S).unwrap ().break_value ().unwrap ().unwrap (), Direction::Down));
        assert! (matches! (validator.validate (Keycode::D).unwrap ().break_value ().unwrap ().unwrap (), Direction::Right));
        assert! (validator.validate (Keycode::X).unwrap ().break_value ().unwrap ().is_none ());
    }

    #[test]
    fn movement_validator_validate () {
        let mut validator = MovementValidator;

        assert! (matches! (validator.validate (Keycode::W).unwrap ().continue_value ().unwrap (), Direction::Up));
        assert! (matches! (validator.validate (Keycode::A).unwrap ().continue_value ().unwrap (), Direction::Left));
        assert! (matches! (validator.validate (Keycode::S).unwrap ().continue_value ().unwrap (), Direction::Down));
        assert! (matches! (validator.validate (Keycode::D).unwrap ().continue_value ().unwrap (), Direction::Right));
        assert! (validator.validate (Keycode::Z).unwrap ().break_value ().unwrap ());
        assert! (!validator.validate (Keycode::X).unwrap ().break_value ().unwrap ());
    }

    #[test]
    fn confirmation_validator_validate () {
        let mut validator = ConfirmationValidator;

        assert! (validator.validate (Keycode::Z).unwrap ().break_value ().unwrap ());
        assert! (!validator.validate (Keycode::X).unwrap ().break_value ().unwrap ());
    }
}
