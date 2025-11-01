/// A PO (Portable Object) file parser that yields (msgid, msgstr) pairs
pub struct PoParser<'a> {
    lines: std::iter::Peekable<std::str::Lines<'a>>,
}

impl<'a> PoParser<'a> {
    /// Create a new PO parser from a string slice
    pub fn new(content: &'a str) -> Self {
        Self {
            lines: content.lines().peekable(),
        }
    }

    /// Parse a multi-line string value (msgid or msgstr)
    fn parse_string_value(&mut self, first_line: &str) -> Result<String, String> {
        let mut result = String::new();

        // Extract the initial quoted string from the first line
        if let Some(quoted) = extract_quoted_string(first_line) {
            result.push_str(&quoted);
        }

        // Continue reading continuation lines
        while let Some(line) = self.lines.peek() {
            let trimmed = line.trim();
            if trimmed.starts_with('"') {
                // It's a continuation line, consume it
                self.lines.next();
                if let Some(quoted) = extract_quoted_string(trimmed) {
                    result.push_str(&quoted);
                }
            } else {
                // Not a continuation line, stop
                break;
            }
        }

        Ok(result)
    }
}

impl<'a> Iterator for PoParser<'a> {
    type Item = Result<(String, String), String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Read next line
            let line = match self.lines.next() {
                Some(l) => l.trim(),
                None => return None, // EOF
            };

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Look for msgid
            if let Some(identifier) = line.strip_prefix("msgid ") {
                let msgid = match self.parse_string_value(identifier) {
                    Ok(s) => s,
                    Err(e) => return Some(Err(e)),
                };

                // Now look for the corresponding msgstr
                loop {
                    let line = match self.lines.next() {
                        Some(l) => l.trim(),
                        None => {
                            return Some(Err("Expected msgstr after msgid".to_string()));
                        }
                    };

                    // Skip empty lines and comments between msgid and msgstr
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    if let Some(value) = line.strip_prefix("msgstr ") {
                        let msgstr = match self.parse_string_value(value) {
                            Ok(s) => s,
                            Err(e) => return Some(Err(e)),
                        };

                        // Skip empty msgid (file header)
                        if msgid.is_empty() {
                            break;
                        }

                        return Some(Ok((msgid, msgstr)));
                    } else {
                        return Some(Err(format!("Expected msgstr, found: {}", line)));
                    }
                }
            }
        }
    }
}

/// Extract a quoted string, handling escape sequences
fn extract_quoted_string(line: &str) -> Option<String> {
    let line = line.trim();

    if !line.starts_with('"') {
        return None;
    }

    let mut result = String::new();
    let chars = line[1..].chars();
    let mut escaped = false;

    for char in chars {
        if escaped {
            match char {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                _ => {
                    result.push('\\');
                    result.push(char);
                }
            }
            escaped = false;
        } else if char == '\\' {
            escaped = true;
        } else if char == '"' {
            // End of string
            return Some(result);
        } else {
            result.push(char);
        }
    }

    // If we get here, the string wasn't properly closed
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_po_entry() {
        let po_content = r#"
msgid "hello"
msgstr "bonjour"

msgid "world"
msgstr "monde"
"#;
        let parser = PoParser::new(po_content);

        let entries: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], ("hello".to_string(), "bonjour".to_string()));
        assert_eq!(entries[1], ("world".to_string(), "monde".to_string()));
    }

    #[test]
    fn test_multiline_po_entry() {
        let po_content = r#"
msgid "This is a "
"multiline string"
msgstr "Ceci est une "
"chaîne multiligne"
"#;
        let parser = PoParser::new(po_content);

        let entries: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0],
            (
                "This is a multiline string".to_string(),
                "Ceci est une chaîne multiligne".to_string()
            )
        );
    }

    #[test]
    fn test_with_comments() {
        let po_content = r#"
# This is a comment
msgid "test"
msgstr "essai"

# Another comment
msgid "example"
msgstr "exemple"
"#;
        let parser = PoParser::new(po_content);

        let entries: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], ("test".to_string(), "essai".to_string()));
        assert_eq!(entries[1], ("example".to_string(), "exemple".to_string()));
    }

    #[test]
    fn test_escape_sequences() {
        let po_content = r#"
msgid "Line 1\nLine 2"
msgstr "Ligne 1\nLigne 2"
"#;
        let parser = PoParser::new(po_content);

        let entries: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0],
            ("Line 1\nLine 2".to_string(), "Ligne 1\nLigne 2".to_string())
        );
    }

    #[test]
    fn test_empty_msgid_header_skipped() {
        let po_content = r#"
msgid ""
msgstr "Project-Id-Version: 1.0\n"

msgid "real_entry"
msgstr "vraie_entrée"
"#;
        let parser = PoParser::new(po_content);

        let entries: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0],
            ("real_entry".to_string(), "vraie_entrée".to_string())
        );
    }
}
