use crate::character::SkillKind;
use crate::common::{Target, ID};
use crate::dynamic::{AppliableKind, StatisticKind, Trigger};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::sync::mpsc::Sender;

pub fn log_then_panic<T: Display> (sender: &Sender<String>, message: T) -> ! {
    sender.send (message.to_string ()).unwrap_or_else (|e| panic! ("{}", e));

    panic! ("{}", message)
}

#[derive (Debug)]
pub enum DanubiaError {
    InvalidLeaderID (ID),
    FailedAddFollower (ID, ID),
    InvalidSkillKind (SkillKind),
    InvalidAppliableKind (AppliableKind),
    InvalidTarget (Target),
    InvalidTrigger (Trigger),
    InvalidStatisticKind (StatisticKind),
}

impl Display for DanubiaError {
    fn fmt (&self, f: &mut Formatter) -> fmt::Result {
        let error: String = match self {
            DanubiaError::InvalidLeaderID (leader_id) => format! ("invalid leader ID {}", leader_id),
            DanubiaError::FailedAddFollower (follower_id, leader_id) => format! ("failed to add follower {} for leader {}", follower_id, leader_id),
            DanubiaError::InvalidSkillKind (kind) => format! ("invalid skill kind {:?}", kind),
            DanubiaError::InvalidAppliableKind (kind) => format! ("invalid appliable kind {:?}", kind),
            DanubiaError::InvalidTarget (target) => format! ("invalid target {:?}", target),
            DanubiaError::InvalidTrigger (trigger) => format! ("invalid trigger {:?}", trigger),
            DanubiaError::InvalidStatisticKind (kind) => format! ("invalid statistic kind {:?}", kind),
        };

        write! (f, "{}", error)
    }
}

impl Error for DanubiaError {}
