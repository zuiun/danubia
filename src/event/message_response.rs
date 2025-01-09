use crate::common::ID;

#[derive (Debug)]
#[derive (Clone, Copy)]
pub enum Message {
    TestAdd,
    TestSubtract,
}

impl Message {
    pub const fn discriminant (&self) -> ID {
        match self {
            Message::TestAdd => 0,
            Message::TestSubtract => 1,
        }
    }
}

#[derive (Debug, Clone)]
pub enum Response {
    TestAdd (u8),
    TestSubtract (u8),
}

impl Response {
    pub const fn discriminant (&self) -> ID {
        match self {
            Response::TestAdd ( .. ) => 0,
            Response::TestSubtract ( .. ) => 1,
        }
    }
}

impl PartialEq for Response {
    fn eq (&self, other: &Self) -> bool {
        self.discriminant () == other.discriminant ()
    }
}
