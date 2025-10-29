use xila::internationalization::Configuration;

fn main() {
    xila::internationalization::generate_translations(&Configuration::default()).unwrap();
}
