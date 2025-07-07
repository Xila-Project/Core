pub trait Cache_trait {
    fn flush_data_cache(&self);
    fn flush_instruction_cache(&self);
}
