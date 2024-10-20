use quote::ToTokens;
use syn::{visit::Visit, ForeignItemFn, ItemType, Signature};

use super::Type_tree::Type_tree_type;

#[derive(Default)]
pub struct LVGL_functions_type {
    Functions: Vec<Signature>,
    Type_tree: Type_tree_type,
    Structures: Vec<String>,
}

impl LVGL_functions_type {
    pub fn Get_signatures(&self) -> Vec<Signature> {
        self.Functions.clone()
    }

    pub fn Get_type_tree(&self) -> &Type_tree_type {
        &self.Type_tree
    }

    pub fn Get_structures(&self) -> &Vec<String> {
        &self.Structures
    }

    fn Filter_function(Signature: &Signature) -> bool {
        let Unauthorized_functions = ["lv_obj_get_display"];

        if Unauthorized_functions.contains(&Signature.ident.to_string().as_str()) {
            return false;
        }

        let Authorized_prefixes = [
            "lv_point_",
            "lv_color",
            "lv_style_",
            "lv_obj_",
            "lv_style_",
            "lv_arc_",
            "lv_coord_",
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
            "lv_span_",
            "lv_spangroup_",
            "lv_textarea_",
            "lv_spinbox_",
            "lv_table_",
            "lv_tabview_",
            "lv_tileview_",
            "lv_subject_",
        ];

        Authorized_prefixes
            .iter()
            .any(|&prefix| Signature.ident.to_string().starts_with(prefix))
    }
}

impl Visit<'_> for LVGL_functions_type {
    fn visit_foreign_item_fn(&mut self, Foreign_item_function: &ForeignItemFn) {
        if Self::Filter_function(&Foreign_item_function.sig) {
            self.Functions.push(Foreign_item_function.sig.clone());
        }
    }

    fn visit_item_type(&mut self, i: &ItemType) {
        self.Type_tree.Insert(
            i.ident.to_token_stream().to_string(),
            i.ty.to_token_stream().to_string(),
        );
    }

    fn visit_item_struct(&mut self, i: &'_ syn::ItemStruct) {
        self.Structures.push(i.ident.to_string());
    }
}
