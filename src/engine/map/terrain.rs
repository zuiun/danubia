use std::fmt;
use crate::engine::common::Modifier;

#[derive (Debug)]
pub struct Terrain {
    modifiers: Vec<Modifier>,
    cost: u8
}

impl Terrain {
    pub const fn new (modifiers: Vec<Modifier>, cost: u8 ) -> Self {
        Self { modifiers, cost }
    }

    pub fn get_modifiers (&self) -> &Vec<Modifier> {
        &self.modifiers
    }

    pub fn get_cost (&self) -> u8 {
        self.cost
    }
}

impl fmt::Display for Terrain {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write! (f, "{}", self.cost)
    }
}

#[cfg (test)]
mod tests {
    use std::rc::Rc;
    use crate::engine::Lists;
    use crate::engine::tests::generate_lists;

    #[test]
    fn terrain_data () {
        let lists: Rc<Lists> = generate_lists ();

        assert_eq! (lists.get_terrain (&0).get_modifiers ().len (), 0);
        assert_eq! (lists.get_terrain (&0).get_cost (), 1);
        assert_eq! (lists.get_terrain (&1).get_modifiers ().len (), 0);
        assert_eq! (lists.get_terrain (&1).get_cost (), 2);
        assert_eq! (lists.get_terrain (&2).get_modifiers ().len (), 0);
        assert_eq! (lists.get_terrain (&2).get_cost (), 0);
    }
}