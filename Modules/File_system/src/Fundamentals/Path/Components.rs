use core::str::Split;

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
    pub fn New(Path: &Path_type) -> Components_type {
        Components_type(Path.As_str().split(Separator))
    }

    pub fn Get_common_components(self, Other: Components_type<'a>) -> usize {
        self.zip(Other).take_while(|(a, b)| a == b).count()
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
mod Tests {

    use super::*;

    #[test]
    fn Test_components() {
        assert_eq!(
            Components_type::New(Path_type::From_str("/a/b/c")).collect::<Vec<_>>(),
            vec![
                Component_type::Root,
                Component_type::Normal("a"),
                Component_type::Normal("b"),
                Component_type::Normal("c")
            ]
        );

        assert_eq!(
            Components_type::New(Path_type::From_str("/a/./b/c")).collect::<Vec<_>>(),
            vec![
                Component_type::Root,
                Component_type::Normal("a"),
                Component_type::Current,
                Component_type::Normal("b"),
                Component_type::Normal("c")
            ]
        );

        assert_eq!(
            Components_type::New(Path_type::From_str("a/b/c")).collect::<Vec<_>>(),
            vec![
                Component_type::Normal("a"),
                Component_type::Normal("b"),
                Component_type::Normal("c")
            ]
        );

        assert_eq!(
            Components_type::New(Path_type::From_str("a/./../b/c")).collect::<Vec<_>>(),
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
