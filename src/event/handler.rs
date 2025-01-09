use super::*;
use crate::collections::CrossJoinMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Handler needs to own all observers
#[derive (Debug)]
pub struct Handler {
    // Safety guarantee: Handler will never borrow_mut Observer
    id_observers: HashMap<ID, Rc<RefCell<dyn Observer>>>,
    message_observers: CrossJoinMap<ID, ID>,
    id: ID,
}

impl Handler {
    #[allow (clippy::new_without_default)]
    pub fn new () -> Self {
        let id_observers: HashMap<ID, Rc<RefCell<dyn Observer>>> = HashMap::new ();
        let message_observers: CrossJoinMap<ID, ID> = CrossJoinMap::new ();
        let id: ID = 0;

        Self { id_observers, message_observers, id }
    }

    pub fn register (&mut self, observer: Rc<RefCell<dyn Observer>>) -> ID {
        let observer_id: ID = self.id;

        observer.borrow ().set_observer_id (observer_id);
        self.id_observers.insert (observer_id, observer);
        self.id += 1;

        observer_id
    }

    pub fn deregister (&mut self, observer_id: &ID) -> bool {
        self.message_observers.remove_second (observer_id);

        self.id_observers.remove (observer_id).is_some ()
    }

    pub fn subscribe (&mut self, observer_id: ID, message_id: ID) -> bool {
        self.message_observers.insert ((message_id, observer_id))
    }

    pub fn unsubscribe (&mut self, observer_id: &ID, message_id: &ID) -> bool {
        self.message_observers.remove (message_id, observer_id)
    }

    pub fn notify (&self, message: Message) -> Vec<Response> {
        match self.message_observers.get_first (&message.discriminant ()) {
            Some (c) => c.iter ().filter_map (|o: &ID|
                self.id_observers.get (o)
                        .unwrap_or_else (|| panic! ("Observer not found for ID {:?}", o))
                        .borrow ()
                        .respond (message)
            ).collect::<Vec<Response>> (),
            None => Vec::new (),
        }
    }

    pub fn reduce_responses (responses: &[Response]) -> &Response {
        assert! (responses.len () == 1);

        responses.first ().expect ("Response not found")
    }
}

#[cfg (test)]
mod tests {
    use super::*;
    use crate::common::ID_UNINITIALISED;
    use std::cell::{Cell, RefCell};
    use std::rc::{Rc, Weak};

    #[derive (Debug)]
    struct Thing {
        handler: Weak<RefCell<Handler>>,
        data: Cell<u8>,
        observer_id: Cell<ID>,
    }

    impl Thing {
        fn new (handler: Weak<RefCell<Handler>>) -> Self {
            let data: Cell<u8> = Cell::new (0);
            let observer_id: Cell<ID> = Cell::new (ID_UNINITIALISED);

            Self { handler, data, observer_id }
        }
    }

    impl Observer for Thing {
        fn respond (&self, message: Message) -> Option<Response> {
            match message {
                Message::TestAdd => {
                    let mut data: u8 = self.data.get ();

                    data += 1;
                    self.data.replace (data);

                    Some (Response::TestAdd (data))
                }
                Message::TestSubtract => {
                    let mut data: u8 = self.data.get ();

                    data -= 1;
                    self.data.replace (data);

                    Some (Response::TestSubtract (data))
                }
            }
        }

        fn set_observer_id (&self, observer_id: ID) -> bool {
            if self.observer_id.get () < ID_UNINITIALISED {
                false
            } else {
                self.observer_id.replace (observer_id);

                true
            }
        }
    }

    impl Subject for Thing {
        fn notify (&self, message: Message) -> Vec<Response> {
            self.handler.upgrade ().unwrap ().borrow ().notify (message)
        }
    }

    fn generate_handler () -> Rc<RefCell<Handler>> {
        let handler = Handler::new ();
        let handler = RefCell::new (handler);

        Rc::new (handler)
    }

    #[allow (clippy::type_complexity)]
    fn generate_things (handler: Rc<RefCell<Handler>>) -> (Rc<RefCell<dyn Observer>>, Rc<RefCell<dyn Observer>>) {
        let thing_0 = Thing::new (Rc::downgrade (&handler));
        let thing_0 = RefCell::new (thing_0);
        let thing_0 = Rc::new (thing_0);
        let thing_1 = Thing::new (Rc::downgrade (&handler));
        let thing_1 = RefCell::new (thing_1);
        let thing_1 = Rc::new (thing_1);

        (thing_0, thing_1)
    }

    #[test]
    fn handler_register () {
        let handler = generate_handler ();
        let (thing_0, thing_1) = generate_things (Rc::clone (&handler));

        // Test empty register
        assert_eq! (handler.borrow_mut ().register (thing_0), 0);
        assert! (handler.borrow ().id_observers.contains_key (&0));
        // Test non-empty register
        assert_eq! (handler.borrow_mut ().register (thing_1), 1);
        assert! (handler.borrow ().id_observers.contains_key (&1));
        assert_eq! (handler.borrow ().id, 2);
    }

    #[test]
    fn handler_deregister () {
        let handler = generate_handler ();
        let (thing_0, _) = generate_things (Rc::clone (&handler));

        // Test empty deregister
        assert! (!handler.borrow_mut ().deregister (&0));
        // Test non-empty deregister
        handler.borrow_mut ().register (thing_0);
        assert! (handler.borrow_mut ().deregister (&0));
        assert! (!handler.borrow ().id_observers.contains_key (&0));
    }

    #[test]
    fn handler_subscribe () {
        let handler = generate_handler ();
        let (thing_0, thing_1) = generate_things (Rc::clone (&handler));

        handler.borrow_mut ().register (thing_0);
        handler.borrow_mut ().register (thing_1);
        // Test empty subscribe
        assert! (handler.borrow_mut ().subscribe (0, 0));
        assert_eq! (handler.borrow ().message_observers.get_first (&0).unwrap ().len (), 1);
        // Test non-empty subscribe
        assert! (handler.borrow_mut ().subscribe (1, 0));
        assert_eq! (handler.borrow ().message_observers.get_first (&0).unwrap ().len (), 2);
        // Test conflicting subscribe
        assert! (!handler.borrow_mut ().subscribe (1, 0));
        assert_eq! (handler.borrow ().message_observers.get_first (&0).unwrap ().len (), 2);
        // Test multiple subscribe
        assert! (handler.borrow_mut ().subscribe (1, 1));
        assert_eq! (handler.borrow ().message_observers.get_first (&0).unwrap ().len (), 2);
        assert_eq! (handler.borrow ().message_observers.get_first (&1).unwrap ().len (), 1);
    }

    #[test]
    fn handler_unsubscribe () {
        let handler = generate_handler ();
        let (thing_0, thing_1) = generate_things (Rc::clone (&handler));

        handler.borrow_mut ().register (thing_0);
        handler.borrow_mut ().register (thing_1);
        // Test empty unsubscribe
        assert! (!handler.borrow_mut ().unsubscribe (&0, &0));
        // Test non-empty unsubscribe
        handler.borrow_mut ().subscribe (0, 0);
        assert! (handler.borrow_mut ().unsubscribe (&0, &0));
        assert! (handler.borrow ().message_observers.get_first (&0).unwrap ().is_empty ());
        // Test conflicting unsubscribe
        handler.borrow_mut ().subscribe (0, 0);
        handler.borrow_mut ().subscribe (1, 0);
        assert! (handler.borrow_mut ().unsubscribe (&0, &0));
        assert_eq! (handler.borrow ().message_observers.get_first (&0).unwrap ().len (), 1);
        // Test multiple unsubscribe
        handler.borrow_mut ().subscribe (0, 0);
        handler.borrow_mut ().subscribe (0, 1);
        assert! (handler.borrow_mut ().unsubscribe (&0, &0));
        assert_eq! (handler.borrow ().message_observers.get_first (&0).unwrap ().len (), 1);
        assert_eq! (handler.borrow ().message_observers.get_first (&1).unwrap ().len (), 1);
    }

    #[test]
    fn handler_notify () {
        let handler = generate_handler ();
        let (thing_0, thing_1) = generate_things (Rc::clone (&handler));

        handler.borrow_mut ().register (thing_0);
        handler.borrow_mut ().register (thing_1);
        // Test empty notify
        let responses: Vec<Response> = handler.borrow ().notify (Message::TestAdd);
        assert! (responses.is_empty ());
        // Test non-empty notify
        handler.borrow_mut ().subscribe (0, 0);
        let responses: Vec<Response> = handler.borrow ().notify (Message::TestAdd);
        assert_eq! (responses.len (), 1);
        assert! (responses.contains (&Response::TestAdd (1)));
        assert! (matches! (responses[0], Response::TestAdd (1)));
        // Test conflicting notify
        handler.borrow_mut ().subscribe (0, 1);
        let responses: Vec<Response> = handler.borrow ().notify (Message::TestSubtract);
        assert_eq! (responses.len (), 1);
        assert! (responses.contains (&Response::TestSubtract (0)));
        // Test multiple notify
        handler.borrow_mut ().subscribe (1, 0);
        let responses: Vec<Response> = handler.borrow ().notify (Message::TestAdd);
        assert_eq! (responses.len (), 2);
        assert! (responses.contains (&Response::TestAdd (2)));
        assert! (responses.contains (&Response::TestAdd (1)));
    }
}
