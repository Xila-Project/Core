use alloc::string::String;

#[derive(Debug, PartialEq)]
pub struct OptionArgument<'a>(&'a [String]);

impl<'a> OptionArgument<'a> {
    pub fn get_option_long(&self, name: &str) -> Option<&'a str> {
        self.0
            .iter()
            .filter_map(|s| s.strip_prefix("--"))
            .map(|s| s.split_once('=').unwrap_or((s, "")))
            .find_map(|(key, value)| if key == name { Some(value) } else { None })
    }

    pub fn get_option_short(&self, name: char) -> Option<&'a str> {
        self.0
            .iter()
            .filter_map(|s| s.strip_prefix('-'))
            .map(|s| s.split_once('=').unwrap_or((s, "")))
            .find_map(|(key, value)| {
                if key.chars().next()? == name {
                    Some(value)
                } else {
                    None
                }
            })
    }

    pub fn get_option(&self, name: &str) -> Option<&'a str> {
        if let Some(first_char) = name.chars().next() {
            self.get_option_long(name)
                .or_else(|| self.get_option_short(first_char))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PositionalArgument<'a> {
    pub value: Option<&'a str>,
    pub options: OptionArgument<'a>,
}

#[derive(Debug, Clone)]
pub struct ArgumentsParser<'a> {
    arguments: &'a [String],
}

impl<'a> ArgumentsParser<'a> {
    pub fn new(arguments: &'a [String]) -> Self {
        Self { arguments }
    }
}

impl<'a> Iterator for ArgumentsParser<'a> {
    type Item = PositionalArgument<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.arguments.is_empty() {
            return None;
        }

        let value = if let Some(item) = self.arguments.first()
            && !item.starts_with('-')
        {
            self.arguments = &self.arguments[1..];
            Some(item.as_str())
        } else {
            None
        };

        let options_end = self
            .arguments
            .iter()
            .position(|arg| !arg.starts_with('-'))
            .unwrap_or(self.arguments.len());

        let options = &self.arguments[..options_end];
        self.arguments = &self.arguments[options_end..];

        Some(PositionalArgument {
            value,
            options: OptionArgument(options),
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec, vec::Vec};
    extern crate std;

    use super::*;

    #[test]
    fn test_empty_arguments() {
        let args: Vec<String> = vec![];
        let mut parser = ArgumentsParser::new(&args);
        assert!(parser.next().is_none());
    }

    #[test]
    fn test_single_positional_argument() {
        let args: Vec<String> = vec!["value1".to_string()];
        let mut parser = ArgumentsParser::new(&args);
        let result = parser.next().unwrap();
        assert_eq!(result.value, Some("value1"));
        assert!(result.options.get_option("test").is_none());
        assert!(parser.next().is_none());
    }

    #[test]
    fn test_positional_with_options() {
        let args: Vec<String> = vec![
            "value1".to_string(),
            "--opt1=test".to_string(),
            "-opt2".to_string(),
        ];
        let mut parser = ArgumentsParser::new(&args);
        let result = parser.next().unwrap();
        assert_eq!(result.value, Some("value1"));
        assert_eq!(result.options.get_option("opt1"), Some("test"));
        assert_eq!(result.options.get_option("opt2"), Some(""));
    }

    #[test]
    fn test_multiple_positional_arguments() {
        let args: Vec<String> = vec![
            "value1".to_string(),
            "value2".to_string(),
            "value3".to_string(),
        ];
        let mut parser = ArgumentsParser::new(&args);
        assert_eq!(
            parser.next(),
            Some(PositionalArgument {
                value: Some("value1"),
                options: OptionArgument(&[]),
            })
        );
        assert_eq!(
            parser.next(),
            Some(PositionalArgument {
                value: Some("value2"),
                options: OptionArgument(&[]),
            })
        );
        assert_eq!(
            parser.next(),
            Some(PositionalArgument {
                value: Some("value3"),
                options: OptionArgument(&[]),
            })
        );
        assert!(parser.next().is_none());
    }

    #[test]
    fn test_options_only() {
        let args: Vec<String> = vec!["--opt1=value".to_string(), "-opt2".to_string()];
        let mut parser = ArgumentsParser::new(&args);
        let result = parser.next().unwrap();
        assert_eq!(result.value, None);
        assert_eq!(result.options.get_option("opt1"), Some("value"));
        assert_eq!(result.options.get_option("opt2"), Some(""));
    }

    #[test]
    fn test_mixed_arguments() {
        let args: Vec<String> = vec![
            "pos1".to_string(),
            "--opt1=val1".to_string(),
            "pos2".to_string(),
            "-opt2".to_string(),
            "--opt3=val3".to_string(),
            "pos3".to_string(),
        ];
        let mut parser = ArgumentsParser::new(&args);

        std::println!("{:?}", parser.clone().collect::<Vec<_>>());

        let first = parser.next().unwrap();
        assert_eq!(first.value, Some("pos1"));
        assert_eq!(first.options.get_option("opt1"), Some("val1"));

        let second = parser.next().unwrap();
        assert_eq!(second.value, Some("pos2"));
        assert_eq!(second.options.get_option("opt2"), Some(""));
        assert_eq!(second.options.get_option("opt3"), Some("val3"));

        let third = parser.next().unwrap();
        assert_eq!(third.value, Some("pos3"));
    }

    #[test]
    fn test_option_not_found() {
        let args: Vec<String> = vec!["--opt1=value".to_string()];
        let mut parser = ArgumentsParser::new(&args);
        let result = parser.next().unwrap();
        assert!(result.options.get_option("nonexistent").is_none());
    }

    #[test]
    fn test_option_with_equals_in_value() {
        let args: Vec<String> = vec!["--url=https://example.com?key=value".to_string()];
        let mut parser = ArgumentsParser::new(&args);
        let result = parser.next().unwrap();
        assert_eq!(
            result.options.get_option("url"),
            Some("https://example.com?key=value")
        );
    }

    #[test]
    fn test_double_dash_prefix() {
        let args: Vec<String> = vec!["--option=test".to_string()];
        let mut parser = ArgumentsParser::new(&args);
        let result = parser.next().unwrap();
        assert_eq!(result.options.get_option("option"), Some("test"));
    }
}
