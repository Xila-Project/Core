fn main() {
    #[cfg(target_os = "espidf")]
    {
        embuild::espidf::sysenv::relay();
        embuild::espidf::sysenv::output(); // Only necessary for building the examples
    }
}
