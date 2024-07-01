use std::{
    mem::size_of,
    ptr::{copy_nonoverlapping, null_mut},
};

use Binding_tool::Bind_function_native;
use Graphics::lvgl::sys::{
    lv_calendar_create, lv_disp_get_default, lv_disp_get_hor_res, lv_disp_get_scr_act,
    lv_disp_get_ver_res, lv_mem_alloc, lv_mem_free, lv_mem_realloc, lv_memset_00,
    lv_obj_allocate_spec_attr, lv_obj_class_t, lv_obj_create, lv_obj_del,
    lv_obj_enable_style_refresh, lv_obj_get_index, lv_obj_mark_layout_as_dirty, lv_obj_set_parent,
    lv_obj_t,
};
use Virtual_machine::{Function_descriptor_type, Function_descriptors, Registrable_trait};

pub struct Graphics_bindings {}

impl Registrable_trait for Graphics_bindings {
    fn Get_functions(&self) -> &[Function_descriptor_type] {
        &Graphics_bindings_functions
    }
}

impl Graphics_bindings {
    pub fn New() -> Self {
        println!("Size of lv_obj_t: {}", size_of::<lv_obj_t>());
        Self {}
    }
}

const Graphics_bindings_functions: [Function_descriptor_type; 2] =
    Function_descriptors!(Create_object_binding, Create_calendar_binding);

#[Bind_function_native(Prefix = "Graphics")]
fn Create_object(Result: &mut lv_obj_t) {
    unsafe {
        let obj = lv_obj_create(null_mut());
    }
}

#[Bind_function_native(Prefix = "Graphics")]
fn Create_calendar(Result: &mut lv_obj_t) {
    unsafe {
        let current_screen = lv_disp_get_scr_act(lv_disp_get_default());
    }
}

#[Bind_function_native(Prefix = "Graphics")]
fn Delete_object(Object: &mut lv_obj_t) {
    unsafe {
        lv_obj_del(Object);
    }
}
