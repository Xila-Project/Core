use quote::ToTokens;
use syn::{visit::Visit, ForeignItemFn, ItemFn, ItemStruct, ItemType, ItemUnion, Signature};

use super::Type_tree::Type_tree_type;

#[derive(Default)]
pub struct LVGL_context {
    signatures: Vec<Signature>,
    definitions: Vec<ItemFn>,
    type_tree: Type_tree_type,
    types: Vec<ItemType>,
    structures: Vec<ItemStruct>,
    unions: Vec<ItemUnion>,
    function_filtering: Option<fn(&Signature) -> bool>,
}

impl LVGL_context {
    pub fn set_function_filtering(&mut self, Function_filtering: Option<fn(&Signature) -> bool>) {
        self.function_filtering = Function_filtering;
    }

    pub fn Get_signatures(&self) -> Vec<Signature> {
        self.signatures.clone()
    }

    pub fn Get_definitions(&self) -> Vec<ItemFn> {
        self.definitions.clone()
    }

    pub fn Get_type_tree(&self) -> &Type_tree_type {
        &self.type_tree
    }

    pub fn Get_types(&self) -> &Vec<ItemType> {
        &self.types
    }

    pub fn Get_structures(&self) -> &Vec<ItemStruct> {
        &self.structures
    }

    pub fn Get_unions(&self) -> &Vec<ItemUnion> {
        &self.unions
    }

    fn Contains_excluded_type(Signature: &Signature) -> bool {
        Signature.inputs.iter().any(|input| match input {
            syn::FnArg::Typed(pat_type) => match &*pat_type.ty {
                syn::Type::Path(type_path) => {
                    let path = type_path.path.to_token_stream().to_string();

                    path.contains("_cb_")
                }
                syn::Type::Ptr(Type_ptr) => {
                    let element = Type_ptr.elem.to_token_stream().to_string();

                    element.contains("lv_event_t")
                        || element.ends_with("_dsc_t")
                        || element.ends_with("_cursor_t")
                        || element.ends_with("_font_t")
                        || element.ends_with("_group_t")
                        || element.ends_with("_layer_t")
                }
                _ => false,
            },
            _ => false,
        }) || match &Signature.output {
            syn::ReturnType::Type(_, type_value) => match &**type_value {
                syn::Type::Path(type_path) => {
                    let path = type_path.path.to_token_stream().to_string();

                    path.contains("_cb_")
                }
                syn::Type::Ptr(Type_ptr) => {
                    let element = Type_ptr.elem.to_token_stream().to_string();

                    element.contains("lv_event_t")
                        || element.ends_with("_dsc_t")
                        || element.ends_with("_cursor_t")
                        || element.ends_with("_font_t")
                        || element.ends_with("_group_t")
                        || element.ends_with("_layer_t")
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn Filter_function(Signature: &Signature) -> bool {
        let unauthorized_functions = [
            "lv_obj_get_display",
            "lv_obj_delete",
            "lv_obj_delete_delayed",
            "lv_obj_delete_async",
            "lv_buttonmatrix_set_map",
        ];

        if unauthorized_functions.contains(&Signature.ident.to_string().as_str()) {
            return false;
        }

        let Authorized_prefixes = [
            "lv_point_",
            "lv_color",
            "lv_label",
            "lv_style_",
            "lv_obj_",
            "lv_style_",
            "lv_arc_",
            "lv_coord_",
            "lv_button_",
            "lv_buttonmatrix_",
            "lv_calendar_",
            "lv_chart_",
            "lv_checkbox_",
            "lv_dropdown_",
            "lv_led_",
            "lv_line_",
            "lv_menu_",
            "lv_msgbox_",
            "lv_roller_",
            "lv_scale_",
            "lv_slider_",
            "lv_textarea_",
            "lv_spinbox_",
            "lv_table_",
            "lv_tabview_",
            "lv_tileview_",
            "lv_subject_",
            "lv_screen_",
            "Window_",
            "Object_",
        ];

        if !Authorized_prefixes
            .iter()
            .any(|&prefix| Signature.ident.to_string().starts_with(prefix))
        {
            return false;
        }

        // Check if on of the function parameter contains a function pointer
        if Self::Contains_excluded_type(Signature) {
            return false;
        }

        true
    }
}

impl Visit<'_> for LVGL_context {
    fn visit_foreign_item_fn(&mut self, Foreign_item_function: &ForeignItemFn) {
        if let Some(filter_function) = self.function_filtering {
            if !filter_function(&Foreign_item_function.sig) {
                return;
            }
        }

        self.signatures.push(Foreign_item_function.sig.clone());
    }

    fn visit_item_type(&mut self, i: &ItemType) {
        self.type_tree.insert(
            i.ident.to_token_stream().to_string(),
            i.ty.to_token_stream().to_string(),
        );

        self.types.push(i.clone());
    }

    fn visit_item_struct(&mut self, i: &'_ ItemStruct) {
        self.structures.push(i.clone());
    }

    fn visit_item_union(&mut self, i: &'_ ItemUnion) {
        self.unions.push(i.clone());
    }

    fn visit_item_fn(&mut self, i: &'_ syn::ItemFn) {
        if let Some(filter_function) = self.function_filtering {
            if !filter_function(&i.sig) {
                return;
            }
        }

        self.signatures.push(i.sig.clone());
        self.definitions.push(i.clone());
    }
}
