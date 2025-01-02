pub mod handler;
pub use self::handler::Handler;

mod message_response;
pub use self::message_response::EVENT_GAME_UNIT_DIE;
pub use self::message_response::EVENT_UNIT_TAKE_DAMAGE;
pub use self::message_response::EVENT_UNIT_ADD_STATUS;
pub use self::message_response::EVENT_UNIT_ADD_APPLIABLE;
pub use self::message_response::EVENT_GRID_FIND_UNITS;
pub use self::message_response::EVENT_GRID_FIND_LOCATIONS;
pub use self::message_response::EVENT_GRID_GET_UNIT_LOCATION;
pub use self::message_response::EVENT_GRID_IS_UNIT_ON_IMPASSABLE;
pub use self::message_response::EVENT_GRID_FIND_UNIT_CITIES;
pub use self::message_response::EVENT_UNIT_GET_STATISTICS;
pub use self::message_response::EVENT_FACTION_IS_MEMBER;
pub use self::message_response::EVENT_UNIT_GET_FACTION_ID;
pub use self::message_response::EVENT_FACTION_ADD_MEMBER;
pub use self::message_response::EVENT_GRID_ADD_STATUS;
pub use self::message_response::EVENT_GRID_TRY_YIELD_APPLIABLE;
pub use self::message_response::EVENT_UNIT_CHANGE_MODIFIER_TERRAIN;
pub use self::message_response::EVENT_GRID_FIND_DISTANCE_BETWEEN;
pub use self::message_response::EVENT_UNIT_TRY_ADD_PASSIVE;
pub use self::message_response::EVENT_reuse_later;
pub use self::message_response::EVENT_FACTION_ADD_FOLLOWER;
pub use self::message_response::EVENT_FACTION_GET_LEADER;
pub use self::message_response::EVENT_FACTION_GET_FOLLOWERS;
pub use self::message_response::EVENT_UNIT_SET_LEADER;
pub use self::message_response::EVENT_UNIT_SEND_PASSIVE;
pub use self::message_response::SUBJECT_UNIT_TYPE;
pub use self::message_response::SUBJECT_GRID_TYPE;
pub use self::message_response::SUBJECT_FACTION_TYPE;
pub use self::message_response::event_iter;
pub use self::message_response::Message;
pub use self::message_response::Response;

use core::fmt::Debug;
use crate::common::ID;

pub trait Observer: Debug {
    /*
     * Responds to message
     * If a response requires mutating internal state, then this uses a Cell or RefCell
     *
     * message: Message = message for update
     *
     * Pre: None
     * Post: Response matches message variant
     * Return: Option<Response> = None -> no response, Some (response) -> response
     */
    fn respond (&self, message: Message) -> Option<Response>;
    /*
     * Sets self's observer ID to observer_id
     * Fails if self's observer ID is already initialised
     * This mutates internal state with a Cell
     *
     * Pre: None
     * Post: None
     * Return: bool = false -> set failed, true -> set succeeded
     */
    fn set_observer_id (&self, observer_id: ID) -> bool;
}

pub trait Subject {
    /*
     * Notifies handler with message
     *
     * Pre: None
     * Post: None
     * Return: Response = ???
     */
    fn notify (&self, message: Message) -> Vec<Response>;
}
