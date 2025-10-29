use std::collections::HashMap;

pub fn extract_translations(
    toml_value: &toml::Value,
    locale: &str,
    fallback: &str,
) -> HashMap<String, String> {
    let mut translations = HashMap::new();
    extract_translations_recursive(
        toml_value,
        locale,
        fallback,
        String::new(),
        &mut translations,
    );
    translations
}

fn extract_translations_recursive(
    toml_value: &toml::Value,
    locale: &str,
    fallback: &str,
    prefix: String,
    translations: &mut HashMap<String, String>,
) {
    if let Some(table) = toml_value.as_table() {
        for (key, value) in table.iter() {
            // Skip keys that start with _
            if key.starts_with('_') {
                continue;
            }

            // Build the current key path
            let current_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            if let Some(value_table) = value.as_table() {
                // Check if this table contains translations (has locale keys)
                let has_locale_key =
                    value_table.contains_key(locale) || value_table.contains_key(fallback);

                if has_locale_key {
                    // This is a translation node - extract the translation
                    let translation =
                        value_table
                            .get(locale)
                            .and_then(|v| v.as_str())
                            .or_else(|| {
                                // Fallback to the fallback locale
                                value_table.get(fallback).and_then(|v| v.as_str())
                            });

                    if let Some(text) = translation {
                        translations.insert(current_key.clone(), text.to_string());
                    }
                } else {
                    // This is a nested structure - recurse into it
                    extract_translations_recursive(
                        value,
                        locale,
                        fallback,
                        current_key,
                        translations,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_translations_simple() {
        let toml_str = r#"
            [greeting]
            en = "Hello"
            fr = "Bonjour"
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let translations = extract_translations(&toml_value, "en", "en");

        assert_eq!(translations.get("greeting"), Some(&"Hello".to_string()));
    }

    #[test]
    fn test_extract_translations_with_fallback() {
        let toml_str = r#"
            [greeting]
            en = "Hello"
            fr = "Bonjour"
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let translations = extract_translations(&toml_value, "de", "en");

        assert_eq!(translations.get("greeting"), Some(&"Hello".to_string()));
    }

    #[test]
    fn test_extract_translations_nested() {
        let toml_str = r#"
            [app.title]
            en = "My App"
            fr = "Mon Application"
            
            [app.menu.file]
            en = "File"
            fr = "Fichier"
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let translations = extract_translations(&toml_value, "en", "en");

        assert_eq!(translations.get("app.title"), Some(&"My App".to_string()));
        assert_eq!(translations.get("app.menu.file"), Some(&"File".to_string()));
    }

    #[test]
    fn test_extract_translations_skip_underscore() {
        let toml_str = r#"
            [_metadata]
            version = "1.0"
            
            [greeting]
            en = "Hello"
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let translations = extract_translations(&toml_value, "en", "en");

        assert!(!translations.contains_key("_metadata.version"));
        assert_eq!(translations.get("greeting"), Some(&"Hello".to_string()));
    }

    #[test]
    fn test_extract_translations_empty() {
        let toml_str = r#"
            [section]
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let translations = extract_translations(&toml_value, "en", "en");

        assert!(translations.is_empty());
    }

    #[test]
    fn test_extract_translations_missing_locale() {
        let toml_str = r#"
            [greeting]
            fr = "Bonjour"
            de = "Hallo"
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let translations = extract_translations(&toml_value, "en", "fr");

        assert_eq!(translations.get("greeting"), Some(&"Bonjour".to_string()));
    }

    #[test]
    fn test_extract_translations_multiple_nested_levels() {
        let toml_str = r#"
            [app.settings.display.theme]
            en = "Theme"
            fr = "Thème"
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let translations = extract_translations(&toml_value, "en", "en");

        assert_eq!(
            translations.get("app.settings.display.theme"),
            Some(&"Theme".to_string())
        );
    }

    #[test]
    fn test_extract_translations_mixed_structure() {
        let toml_str = r#"
            [root]
            en = "Root"
            
            [nested.item]
            en = "Item"
            
            [nested.another.deep]
            en = "Deep"
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let translations = extract_translations(&toml_value, "en", "en");

        assert_eq!(translations.len(), 3);
        assert_eq!(translations.get("root"), Some(&"Root".to_string()));
        assert_eq!(translations.get("nested.item"), Some(&"Item".to_string()));
        assert_eq!(
            translations.get("nested.another.deep"),
            Some(&"Deep".to_string())
        );
    }
}
