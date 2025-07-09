use core::str::Split;

use super::{Path_type, SEPARATOR};

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
            "" => Component_type::Root,
            "/" => Component_type::Root,
            "." => Component_type::Current,
            ".." => Component_type::Parent,
            _ => Component_type::Normal(item),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Components_type<'a>(Split<'a, char>);

impl<'a> Components_type<'a> {
    pub fn new(path: &Path_type) -> Components_type {
        Components_type(path.as_str().split(SEPARATOR))
    }

    pub fn get_common_components(self, other: Components_type<'a>) -> usize {
        self.zip(other).take_while(|(a, b)| a == b).count()
    }
}

impl<'a> Iterator for Components_type<'a> {
    type Item = Component_type<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Component_type::from)
    }
}

impl DoubleEndedIterator for Components_type<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(Component_type::from)
    }
}

#[cfg(test)]
mod tests {

    use alloc::{vec, vec::Vec};

    use super::*;

    #[test]
    fn test_components() {
        assert_eq!(
            Components_type::new(Path_type::from_str("/a/b/c")).collect::<Vec<_>>(),
            vec![
                Component_type::Root,
                Component_type::Normal("a"),
                Component_type::Normal("b"),
                Component_type::Normal("c")
            ]
        );

        assert_eq!(
            Components_type::new(Path_type::from_str("/a/./b/c")).collect::<Vec<_>>(),
            vec![
                Component_type::Root,
                Component_type::Normal("a"),
                Component_type::Current,
                Component_type::Normal("b"),
                Component_type::Normal("c")
            ]
        );

        assert_eq!(
            Components_type::new(Path_type::from_str("a/b/c")).collect::<Vec<_>>(),
            vec![
                Component_type::Normal("a"),
                Component_type::Normal("b"),
                Component_type::Normal("c")
            ]
        );

        assert_eq!(
            Components_type::new(Path_type::from_str("a/./../b/c")).collect::<Vec<_>>(),
            vec![
                Component_type::Normal("a"),
                Component_type::Current,
                Component_type::Parent,
                Component_type::Normal("b"),
                Component_type::Normal("c")
            ]
        );
    }
}
