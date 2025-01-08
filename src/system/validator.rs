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
        let input: char = input.chars ().next ().unwrap_or_else (|| panic! ("Invalid input {}", input));

        match input {
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

pub struct UnitValidator<'a> {
    prompt: &'static str,
    unit_ids: &'a[ID],
}

impl<'a> UnitValidator<'a> {
    pub fn new (unit_ids: &'a[ID]) -> Self {
        let prompt: &str = "Enter unit ID (#)";

        Self { prompt, unit_ids }
    }
}

impl Validator<ID> for UnitValidator<'_> {
    fn validate (&self, input: &str) -> Result<Option<ID>, String> {
        let input: ID = input.parse ().unwrap_or_else (|e| panic! ("{}", e));

        if self.unit_ids.contains (&input) {
            Ok (Some (input))
        } else {
            Ok (None)
        }
    }

    fn get_prompt (&self) -> &str {
        self.prompt
    }
}

pub struct LocationValidator<'a> {
    prompt: &'static str,
    locations: &'a[Location]
}

impl<'a> LocationValidator<'a> {
    pub fn new (locations: &'a[Location]) -> Self {
        let prompt: &str = "Enter comma-separated location (row, column: #, #)";

        Self { prompt, locations }
    }
}

impl Validator<Location> for LocationValidator<'_> {
    fn validate (&self, input: &str) -> Result<Option<Location>, String> {
        let mut input = input.split (",");
        let i: usize = input.next ().unwrap_or_else (|| panic! ("Invalid input"))
                .trim ().parse ().unwrap_or_else (|e| panic! ("{}", e));
        let j: usize = input.next ().unwrap_or_else (|| panic! ("Invalid input"))
                .trim ().parse ().unwrap_or_else (|e| panic! ("{}", e));

        if self.locations.contains (&(i, j)) {
            Ok (Some ((i, j)))
        } else {
            Ok (None)
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
        let prompt: &str = "Enter direction ([u]p/[r]ight/[l]eft/[d]own) or stop (x)";

        Self { prompt }
    }
}

impl Validator<Direction> for DirectionValidator {
    fn validate (&self, input: &str) -> Result<Option<Direction>, String> {
        let input: char = input.chars ().next ().unwrap_or_else (|| panic! ("Invalid input {}", input));

        match input {
            'u' => Ok (Some (Direction::Up)),
            'r' => Ok (Some (Direction::Right)),
            'l' => Ok (Some (Direction::Left)),
            'd' => Ok (Some (Direction::Down)),
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
    fn unit_validator_validate () {
        let validator = UnitValidator::new (&[0, 1]);

        // Test empty validate
        assert! (validator.validate ("3").unwrap ().is_none ());
        // Test non-empty validate
        assert! (matches! (validator.validate ("0").unwrap ().unwrap (), 0));
        assert! (matches! (validator.validate ("1").unwrap ().unwrap (), 1));
    }

    #[test]
    fn location_validator_validate () {
        let validator = LocationValidator::new (&[(0, 0), (0, 1)]);

        // Test empty validate
        assert! (validator.validate ("1, 0").unwrap ().is_none ());
        // Test non-empty validate
        assert! (matches! (validator.validate ("0, 0").unwrap ().unwrap (), (0, 0)));
        assert! (matches! (validator.validate ("0, 1").unwrap ().unwrap (), (0, 1)));
    }

    #[test]
    fn direction_validate () {
        let validator = DirectionValidator::new ();

        // Test cancel validate
        assert! (validator.validate ("x").unwrap ().is_none ());
        // Test normal validate
        assert! (matches! (validator.validate ("u").unwrap ().unwrap (), Direction::Up));
        assert! (matches! (validator.validate ("r").unwrap ().unwrap (), Direction::Right));
        assert! (matches! (validator.validate ("l").unwrap ().unwrap (), Direction::Left));
        assert! (matches! (validator.validate ("d").unwrap ().unwrap (), Direction::Down));
    }

    #[test]
    fn confirmation_validate () {
        let validator = ConfirmationValidator::new ();

        assert! (validator.validate ("z").unwrap ().unwrap ());
        assert! (!validator.validate ("x").unwrap ().unwrap ());
    }
}
