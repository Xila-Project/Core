use wamr_sys::wasm_function_inst_t;

pub struct Function_type {
    Function_instance: wasm_function_inst_t,
}

impl Function_type {
    pub fn New(Function_instance: wasm_function_inst_t) -> Self {
        Self { Function_instance }
    }
}
