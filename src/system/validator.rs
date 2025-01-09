use super::Action;
use crate::common::ID;
use crate::map::{Direction, Location};

pub trait Validator<T> {
    fn validate (&self, input: &str) -> Result<Option<T>, String>;
    fn get_prompt (&self) -> &str;
}

#[derive (Debug)]
pub struct ActionValidator {
    prompt: &'static str,
}

impl ActionValidator {
    pub fn new () -> Self {
        let prompt: &str = "Enter move (q), switch weapon (w), wait (e), attack (a), skill (s), or magic (d)";

        Self { prompt }
    }
}

impl Validator<Action> for ActionValidator {
    fn validate (&self, input: &str) -> Result<Option<Action>, String> {
        let action: char = input.chars ().next ().unwrap_or_else (|| panic! ("Invalid input {}", input));

        match action {
            'a' => Ok (Some (Action::Attack)),
            'w' => Ok (Some (Action::Weapon)),
            's' => Ok (Some (Action::Skill)),
            'd' => Ok (Some (Action::Magic)),
            'q' => Ok (Some (Action::Move)),
            'e' => Ok (Some (Action::Wait)),
            _ => Err (String::from ("Invalid input")),
        }
    }

    fn get_prompt (&self) -> &str {
        self.prompt
    }
}

impl Default for ActionValidator {
    fn default () -> Self {
        Self::new ()
    }
}

pub struct IDValidator<'a> {
    prompt: &'static str,
    ids: &'a [ID],
}

impl<'a> IDValidator<'a> {
    pub fn new (ids: &'a [ID]) -> Self {
        let prompt: &str = "Enter ID (#) or cancel (x)";

        Self { prompt, ids }
    }
}

impl Validator<ID> for IDValidator<'_> {
    fn validate (&self, input: &str) -> Result<Option<ID>, String> {
        let cancel: char = input.chars ().next ().unwrap_or_else (|| panic! ("Invalid input {}", input));

        if cancel == 'x' {
            Ok (None)
        } else {
            let id: ID = input.parse ().unwrap_or_else (|e| panic! ("{}", e));

            if self.ids.contains (&id) {
                Ok (Some (id))
            } else {
                Err (String::from ("Invalid ID"))
            }
        }
    }

    fn get_prompt (&self) -> &str {
        self.prompt
    }
}

pub struct LocationValidator<'a> {
    prompt: &'static str,
    locations: &'a [Location],
}

impl<'a> LocationValidator<'a> {
    pub fn new (locations: &'a [Location]) -> Self {
        let prompt: &str = "Enter comma-separated location (row, column: #, #) or cancel (x)";

        Self { prompt, locations }
    }
}

impl Validator<Location> for LocationValidator<'_> {
    fn validate (&self, input: &str) -> Result<Option<Location>, String> {
        let cancel: char = input.chars ().next ().unwrap_or_else (|| panic! ("Invalid input {}", input));

        if cancel == 'x' {
            Ok (None)
        } else {
            let mut input = input.split (",");
            let i: usize = input.next ().unwrap_or_else (|| panic! ("Invalid input"))
                    .trim ().parse ().unwrap_or_else (|e| panic! ("{}", e));
            let j: usize = input.next ().unwrap_or_else (|| panic! ("Invalid input"))
                    .trim ().parse ().unwrap_or_else (|e| panic! ("{}", e));

            if self.locations.contains (&(i, j)) {
                Ok (Some ((i, j)))
            } else {
                Err (String::from ("Invalid location"))
            }
        }
    }

    fn get_prompt (&self) -> &str {
        self.prompt
    }
}

pub struct DirectionValidator {
    prompt: &'static str,
}

impl DirectionValidator {
    pub fn new () -> Self {
        let prompt: &str = "Enter direction (w/a/s/d) or cancel (x)";

        Self { prompt }
    }
}

impl Validator<Direction> for DirectionValidator {
    fn validate (&self, input: &str) -> Result<Option<Direction>, String> {
        let input: char = input.chars ().next ().unwrap_or_else (|| panic! ("Invalid input {}", input));

        match input {
            'w' => Ok (Some (Direction::Up)),
            'd' => Ok (Some (Direction::Right)),
            'a' => Ok (Some (Direction::Left)),
            's' => Ok (Some (Direction::Down)),
            'x' => Ok (None),
            _ => Err (String::from ("Invalid input")),
        }
    }

    fn get_prompt (&self) -> &str {
        self.prompt
    }
}

impl Default for DirectionValidator {
    fn default () -> Self {
        Self::new ()
    }
}

pub struct ConfirmationValidator {
    prompt: &'static str,
}

pub struct MovementValidator {
    prompt: &'static str,
}

impl MovementValidator {
    pub fn new () -> Self {
        let prompt: &str = "Enter direction (w/a/s/d), confirm (z), or cancel (x)";

        Self { prompt }
    }
}

impl Validator<Direction> for MovementValidator {
    fn validate (&self, input: &str) -> Result<Option<Direction>, String> {
        let input: char = input.chars ().next ().unwrap_or_else (|| panic! ("Invalid input {}", input));

        match input {
            'w' => Ok (Some (Direction::Up)),
            'd' => Ok (Some (Direction::Right)),
            'a' => Ok (Some (Direction::Left)),
            's' => Ok (Some (Direction::Down)),
            'z' => Ok (Some (Direction::Length)),
            'x' => Ok (None),
            _ => Err (String::from ("Invalid input")),
        }
    }

    fn get_prompt (&self) -> &str {
        self.prompt
    }
}

impl Default for MovementValidator {
    fn default () -> Self {
        Self::new ()
    }
}

impl ConfirmationValidator {
    pub fn new () -> Self {
        let prompt: &str = "Enter confirm (z) or cancel (x)";

        Self { prompt }
    }
}

impl Validator<bool> for ConfirmationValidator {
    fn validate (&self, input: &str) -> Result<Option<bool>, String> {
        let input: char = input.chars ().next ().unwrap_or_else (|| panic! ("Invalid input {}", input));

        match input {
            'z' => Ok (Some (true)),
            'x' => Ok (Some (false)),
            _ => Err (String::from ("Invalid input")),
        }
    }

    fn get_prompt (&self) -> &str {
        self.prompt
    }
}

impl Default for ConfirmationValidator {
    fn default () -> Self {
        Self::new ()
    }
}

#[cfg (test)]
mod tests {
    use super::*;

    #[test]
    fn action_validator_validate () {
        let validator = ActionValidator::new ();

        assert! (matches! (validator.validate ("a").unwrap ().unwrap (), Action::Attack));
        assert! (matches! (validator.validate ("w").unwrap ().unwrap (), Action::Weapon));
        assert! (matches! (validator.validate ("s").unwrap ().unwrap (), Action::Skill));
        assert! (matches! (validator.validate ("d").unwrap ().unwrap (), Action::Magic));
        assert! (matches! (validator.validate ("q").unwrap ().unwrap (), Action::Move));
        assert! (matches! (validator.validate ("e").unwrap ().unwrap (), Action::Wait));
    }

    #[test]
    fn id_validator_validate () {
        let validator = IDValidator::new (&[0, 1]);

        // Test cancel validate
        assert! (validator.validate ("x").unwrap ().is_none ());
        // Test empty validate
        assert! (validator.validate ("3").is_err ());
        // Test non-empty validate
        assert! (matches! (validator.validate ("0").unwrap ().unwrap (), 0));
        assert! (matches! (validator.validate ("1").unwrap ().unwrap (), 1));
    }

    #[test]
    fn location_validator_validate () {
        let validator = LocationValidator::new (&[(0, 0), (0, 1)]);

        // Test cancel validate
        assert! (validator.validate ("x").unwrap ().is_none ());
        // Test empty validate
        assert! (validator.validate ("1, 0").is_err ());
        // Test non-empty validate
        assert! (matches! (validator.validate ("0, 0").unwrap ().unwrap (), (0, 0)));
        assert! (matches! (validator.validate ("0, 1").unwrap ().unwrap (), (0, 1)));
    }

    #[test]
    fn direction_validator_validate () {
        let validator = DirectionValidator::new ();

        // Test cancel validate
        assert! (validator.validate ("x").unwrap ().is_none ());
        // Test normal validate
        assert! (matches! (validator.validate ("w").unwrap ().unwrap (), Direction::Up));
        assert! (matches! (validator.validate ("d").unwrap ().unwrap (), Direction::Right));
        assert! (matches! (validator.validate ("a").unwrap ().unwrap (), Direction::Left));
        assert! (matches! (validator.validate ("s").unwrap ().unwrap (), Direction::Down));
    }

    #[test]
    fn movement_validator_validate () {
        let validator = MovementValidator::new ();

        // Test cancel validate
        assert! (validator.validate ("x").unwrap ().is_none ());
        // Test confirm validate
        assert! (matches! (validator.validate ("z").unwrap ().unwrap (), Direction::Length));
        // Test normal validate
        assert! (matches! (validator.validate ("w").unwrap ().unwrap (), Direction::Up));
        assert! (matches! (validator.validate ("d").unwrap ().unwrap (), Direction::Right));
        assert! (matches! (validator.validate ("a").unwrap ().unwrap (), Direction::Left));
        assert! (matches! (validator.validate ("s").unwrap ().unwrap (), Direction::Down));
    }

    #[test]
    fn confirmation_validator_validate () {
        let validator = ConfirmationValidator::new ();

        assert! (validator.validate ("z").unwrap ().unwrap ());
        assert! (!validator.validate ("x").unwrap ().unwrap ());
    }
}
