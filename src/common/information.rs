use std::fmt;

#[derive (Debug)]
pub struct Information {
    name: &'static str,
    descriptions: [&'static str; 3],
    description_current: usize,
}

impl Information {
    pub const fn new (name: &'static str, descriptions: [&'static str; 3], description_current: usize) -> Self {
        Self { name, descriptions, description_current }
    }

    pub fn get_name (&self) -> &str {
        &self.name
    }

    pub fn get_description (&self) -> &str {
        &self.descriptions[self.description_current]
    }
}

impl fmt::Display for Information {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}\n{}", self.name, self.descriptions[self.description_current])
    }
}
