use crate::common::ID;
use std::fmt::Debug;

mod handler;
pub use self::handler::Handler;
mod message_response;
pub use self::message_response::Message;
pub use self::message_response::Response;

pub trait Observer: Debug {
    /*
     * Responds to message
     * If a response requires mutating internal state, then this uses a Cell or RefCell
     * Avoid using this for internal mutability unless justifiable
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
