use super::{Path_type, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component_type<'a> {
    Root,
    Current,
    Parent,
    Normal(&'a str),
}

impl<'a> From<&'a str> for Component_type<'a> {
    fn from(item: &'a str) -> Self {
        match item {
            "/" => Component_type::Root,
            "." => Component_type::Current,
            ".." => Component_type::Parent,
            _ => Component_type::Normal(item),
        }
    }
}

pub struct Components_type<'a> {
    Components: Vec<Component_type<'a>>,
    Front: usize,
    Back: usize,
}

impl<'a> Components_type<'a> {
    pub fn New<'b>(Path: &'b Path_type) -> Components_type<'b> {
        let mut Components: Vec<Component_type<'b>> = Path
            .As_str()
            .split(Separator)
            .map(Component_type::from)
            .collect();

        if Path.Is_absolute() {
            Components.insert(0, Component_type::Root); // TODO : Find a way to avoid this relocation.
        }

        let Components_length = Components.len();

        Components_type {
            Components,
            Front: 0,
            Back: Components_length,
        }
    }
}

impl<'a> Iterator for Components_type<'a> {
    type Item = Component_type<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.Front += 1;
        self.Components.get(self.Front - 1).cloned()
    }
}

impl<'a> DoubleEndedIterator for Components_type<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.Back -= 1;
        self.Components.get(self.Back).cloned()
    }
}
