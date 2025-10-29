/// Helper function to generate translations and load them for testing
#[cfg(feature = "std")]
fn generate_and_load_translations(locale: &str, fallback: &str) -> String {
    use internationalization::{Configuration, generate_translations};
    use std::fs;
    use std::path::PathBuf;

    static FILE_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    let input_path = PathBuf::from("tests/test_locales.toml");
    let file_count = FILE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let output_path = PathBuf::from(format!("test_output_{locale}_{file_count}.generated.rs"));

    let config = Configuration {
        input_path,
        locale: locale.to_string(),
        fallback: fallback.to_string(),
        output_path: output_path.clone(),
    };

    generate_translations(&config).expect("Failed to generate translations");

    let string = fs::read_to_string(&output_path).expect("Failed to read generated file");

    fs::remove_file(&output_path).unwrap();

    string
}

/// Test basic translation generation with English locale
#[test]
#[cfg(feature = "std")]
fn test_generate_translations_english() {
    let output = generate_and_load_translations("en", "en");

    // Test that macros are generated
    assert!(output.contains("macro_rules ! app__name"));
    assert!(output.contains(r#""Test Application""#));

    // Test nested translations
    assert!(output.contains("macro_rules ! app__menu__title"));
    assert!(output.contains(r#""Menu""#));
    assert!(output.contains("macro_rules ! app__menu__file_menu__new"));
    assert!(output.contains(r#""New""#));

    // Test messages
    assert!(output.contains("macro_rules ! messages__welcome"));
    assert!(output.contains(r#""Welcome!""#));

    // Test error messages
    assert!(output.contains("macro_rules ! messages__errors__not_found"));
    assert!(output.contains(r#""Not found""#));

    // Test user section
    assert!(output.contains("macro_rules ! user__profile"));
    assert!(output.contains(r#""Profile""#));

    // Test hyphenated keys are normalized
    assert!(output.contains("macro_rules ! special__chars_test__hyphen_key"));
    assert!(output.contains(r#""Hyphen key value""#));

    // Test only-english section
    assert!(output.contains("macro_rules ! only_english__test"));
    assert!(output.contains(r#""English only text""#));

    // Test internal constants
    assert!(output.contains(r#"const __INTERNATIONALIZATION_LOCALE"#));
    assert!(output.contains(r#"const __INTERNATIONALIZATION_FALLBACK"#));
}

/// Test translation generation with French locale
#[test]
#[cfg(feature = "std")]
fn test_generate_translations_french() {
    let output = generate_and_load_translations("fr", "en");

    // Test top-level translations
    assert!(output.contains("macro_rules ! app__name"));
    assert!(output.contains(r#""Application de Test""#));

    // Test nested translations
    assert!(output.contains("macro_rules ! app__menu__file_menu__new"));
    assert!(output.contains(r#""Nouveau""#));
    assert!(output.contains("macro_rules ! app__menu__file_menu__save"));
    assert!(output.contains(r#""Enregistrer""#));

    // Test messages
    assert!(output.contains("macro_rules ! messages__welcome"));
    assert!(output.contains(r#""Bienvenue!""#));

    // Test error messages
    assert!(output.contains("macro_rules ! messages__errors__not_found"));
    assert!(output.contains(r#""Non trouvé""#));

    // Test user section
    assert!(output.contains("macro_rules ! user__profile"));
    assert!(output.contains(r#""Profil""#));

    // Test fallback for missing French translation (only-english section)
    assert!(output.contains("macro_rules ! only_english__test"));
    assert!(output.contains(r#""English only text""#));
}

/// Test translation generation with Spanish locale
#[test]
#[cfg(feature = "std")]
fn test_generate_translations_spanish() {
    let output = generate_and_load_translations("es", "en");

    // Test top-level translations
    assert!(output.contains("macro_rules ! app__name"));
    assert!(output.contains(r#""Aplicación de Prueba""#));

    // Test nested translations
    assert!(output.contains("macro_rules ! app__menu__title"));
    assert!(output.contains(r#""Menú""#));
    assert!(output.contains("macro_rules ! app__menu__file_menu__new"));
    assert!(output.contains(r#""Nuevo""#));

    // Test messages
    assert!(output.contains("macro_rules ! messages__welcome"));
    assert!(output.contains(r#""¡Bienvenido!""#));

    // Test error messages
    assert!(output.contains("macro_rules ! messages__errors__not_found"));
    assert!(output.contains(r#""No encontrado""#));

    // Test user section
    assert!(output.contains("macro_rules ! user__profile"));
    assert!(output.contains(r#""Perfil""#));

    // Test fallback for missing Spanish translation
    assert!(output.contains("macro_rules ! only_english__test"));
    assert!(output.contains(r#""English only text""#));
}

/// Test that translation keys are normalized correctly
#[test]
#[cfg(feature = "std")]
fn test_key_normalization() {
    let output = generate_and_load_translations("en", "en");

    // Keys with dots should be converted to double underscores and lowercased
    // "app.name" -> "app__name"
    assert!(output.contains("macro_rules ! app__name"));

    // Keys with hyphens should be converted to single underscores and lowercased
    // "messages.errors.not-found" -> "messages__errors__not_found"
    assert!(output.contains("macro_rules ! messages__errors__not_found"));

    // Keys with both dots and hyphens should work
    // "special.chars-test.hyphen-key" -> "special__chars_test__hyphen_key"
    assert!(output.contains("macro_rules ! special__chars_test__hyphen_key"));
}

/// Test fallback behavior when translation is missing
#[test]
#[cfg(feature = "std")]
fn test_fallback_behavior() {
    let output = generate_and_load_translations("de", "en");

    // All translations should fall back to English
    assert!(output.contains("macro_rules ! app__name"));
    assert!(output.contains(r#""Test Application""#));
    assert!(output.contains("macro_rules ! messages__welcome"));
    assert!(output.contains(r#""Welcome!""#));
    assert!(output.contains("macro_rules ! user__profile"));
    assert!(output.contains(r#""Profile""#));
}

/// Test that private keys (starting with _) are ignored
#[test]
#[cfg(feature = "std")]
fn test_private_keys_ignored() {
    let output = generate_and_load_translations("en", "en");

    // Private key "_internal" should not be generated
    assert!(!output.contains("macro_rules ! _internal"));
    assert!(!output.contains("macro_rules ! internal") || !output.contains(r#""Internal""#));

    // Public keys should still be accessible
    assert!(output.contains("macro_rules ! app__name"));
}

/// Test that generated macros are properly defined
#[test]
#[cfg(feature = "std")]
fn test_macro_definition() {
    let output = generate_and_load_translations("en", "en");

    // All macros should be declared with macro_rules!
    assert!(output.contains("macro_rules !"));

    // Macros should support both string and C string variants
    assert!(output.contains("() => {"));
    assert!(output.contains("(c) => {"));
}

/// Test deep nesting of translation keys
#[test]
#[cfg(feature = "std")]
fn test_deep_nesting() {
    let output = generate_and_load_translations("en", "en");

    // Test deeply nested keys work correctly
    assert!(output.contains("macro_rules ! app__menu__file_menu__new"));
    assert!(output.contains("macro_rules ! app__menu__edit_menu__undo"));
    assert!(output.contains("macro_rules ! messages__errors__access_denied"));
}

/// Test that multiple locales can be generated without conflict
#[test]
#[cfg(feature = "std")]
fn test_multiple_locales() {
    let en_output = generate_and_load_translations("en", "en");
    let fr_output = generate_and_load_translations("fr", "en");
    let es_output = generate_and_load_translations("es", "en");

    // Verify all three have different values for the same key
    assert!(en_output.contains(r#""Test Application""#));
    assert!(fr_output.contains(r#""Application de Test""#));
    assert!(es_output.contains(r#""Aplicación de Prueba""#));
}

/// Test error handling for missing file
#[test]
#[cfg(feature = "std")]
fn test_missing_file_error() {
    let input_path = PathBuf::from("tests/nonexistent.toml");
    let output_path = PathBuf::from("target/test_output_error.rs");

    let config = Configuration {
        input_path,
        locale: "en".to_string(),
        fallback: "en".to_string(),
        output_path,
    };

    let result = generate_translations(&config);
    assert!(result.is_err());
}

/// Test that the translate macro works correctly (compile-time check)
#[test]
#[cfg(feature = "std")]
fn test_translate_macro() {
    // Generate translations for this test
    let input_path = PathBuf::from("tests/test_locales.toml");
    let output_path = PathBuf::from("test_translate_macro.rs");

    let config = Configuration {
        input_path,
        locale: "en".to_string(),
        fallback: "en".to_string(),
        output_path: output_path.clone(),
    };

    generate_translations(&config).expect("Failed to generate translations");

    // Load the generated file
    let generated = fs::read_to_string(&output_path).expect("Failed to read generated file");

    // Verify the macro names that would be generated
    // "app.name" should generate app__name macro
    assert!(generated.contains("macro_rules ! app__name"));

    // "messages.errors.not-found" should generate messages__errors__not_found macro
    assert!(generated.contains("macro_rules ! messages__errors__not_found"));

    // Clean up
    fs::remove_file(&output_path).unwrap();
}
